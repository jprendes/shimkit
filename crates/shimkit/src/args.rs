use std::collections::HashMap;
use std::env::current_exe;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher as _};
use std::io::{stdout, IsTerminal, Result as IoResult, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context as _, Result};
use os_str_bytes::OsStrBytesExt as _;
use prost::Message;
use shimkit_types::task::KeyValue;
use trapeze::{service, Client, Server, ServerHandle};

use crate::event::EventPublisher;
use crate::fs::dev_null;
use crate::stdio::Duplicate as _;
use crate::sys::CONTAINERD_DEFAULT_ADDRESS;
use crate::types::sandbox::Sandbox;
use crate::types::task::{CleanupRequest, Task};
use crate::utils::ToLossyString;

pub struct Arguments {
    // the id of the container
    pub id: String,

    // the namespace for the container
    pub namespace: String,

    // the address of containerd's ttrpc API socket
    pub ttrpc_address: String,

    // the address of containerd's grpc API socket
    pub grpc_address: String,

    // the binary path to publish events back to containerd (default: containerd)
    pub publish_binary: PathBuf,

    // enable debug output in logs
    pub debug: bool,

    pub(crate) action: String,
    pub(crate) rest: Vec<String>,
    pub(crate) bundle: PathBuf,
    pub(crate) shim_name: OsString,
    pub(crate) stdout: File,
}

impl std::fmt::Debug for Arguments {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Arguments")
            .field("id", &self.id)
            .field("namespace", &self.namespace)
            .field("publish_binary", &self.publish_binary)
            .field("grpc_address", &self.grpc_address)
            .field("ttrpc_address", &self.ttrpc_address)
            .field("debug", &self.debug)
            .finish()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            id: Default::default(),
            namespace: Default::default(),
            ttrpc_address: Default::default(),
            grpc_address: Default::default(),
            publish_binary: Default::default(),
            debug: Default::default(),
            action: Default::default(),
            rest: Default::default(),
            bundle: Default::default(),
            shim_name: Default::default(),
            stdout: dev_null().unwrap(),
        }
    }
}

impl Arguments {
    pub(crate) fn to_args_vec(&self, action: &'static OsStr) -> Vec<&OsStr> {
        let mut args: Vec<&OsStr> = vec![
            "-id".as_ref(),
            self.id.as_ref(),
            "-namespace".as_ref(),
            self.namespace.as_ref(),
            "-address".as_ref(),
            self.grpc_address.as_ref(),
            "-publish-binary".as_ref(),
            self.publish_binary.as_ref(),
        ];
        if self.debug {
            args.push("-debug".as_ref());
        }
        args.push(action.as_ref());
        args.extend(self.rest.iter().map(AsRef::<OsStr>::as_ref));
        args
    }

    pub fn is_interactive(&self) -> bool {
        self.stdout.is_terminal()
    }

    pub async fn serve(
        self,
        address: impl AsRef<Path>,
        server: impl Sandbox + Task,
    ) -> Result<ServerHandle> {
        match self.action.as_str() {
            "version" => {
                let mut stdout = self.stdout;
                let result = server.version(()).await?;
                writeln!(stdout, "{}:", result.executable)?;
                for KeyValue { key, value } in result.info {
                    writeln!(stdout, "  {key}: {value}")?;
                }
                Ok(ServerHandle::new())
            }
            "delete" => {
                let mut stdout = self.stdout;
                let req = CleanupRequest {
                    bundle: self.bundle.to_lossy_string(),
                };
                let result = server.cleanup(req).await?.encode_to_vec();
                stdout.write_all(&result)?;
                Ok(ServerHandle::new())
            }
            "daemon" => {
                let address = address.as_ref().display().to_string();

                #[cfg(unix)]
                let address = format!("unix://{address}");

                let mut stdout = self.stdout;
                writeln!(stdout, "{}", address)?;

                if Client::connect(&address).await.is_ok() {
                    // a server is already running on that address
                    return Ok(ServerHandle::new());
                }

                let handle = Server::new()
                    .register(service!(server : Sandbox + Task))
                    .bind(&address)
                    .await
                    .context("Error binding listener")?;

                Ok(handle)
            }
            action => bail!("Unsupported action `{action}`"),
        }
    }
}

fn shim_name() -> OsString {
    if let Some(name) = current_exe().unwrap_or_default().file_stem() {
        name.strip_prefix("containerd-shim-")
            .unwrap_or(name)
            .to_owned()
    } else {
        OsString::from("unknown")
    }
}

fn socket_address(containerd_socket: impl AsRef<Path>, id: impl AsRef<OsStr>) -> PathBuf {
    let containerd_socket = containerd_socket.as_ref();
    let (_, extension) = containerd_socket
        .file_name()
        .unwrap_or_default()
        .split_once(".")
        .unwrap_or_default();
    let mut name = OsString::from("containerd-shim-");
    name.push(id);
    containerd_socket
        .with_file_name(name)
        .with_extension(extension)
}

impl Arguments {
    pub fn socket_address(&self, id: impl Hash) -> PathBuf {
        let id = {
            let mut hasher = DefaultHasher::new();
            (&self.namespace, id).hash(&mut hasher);
            hasher.finish()
        };

        self.socket_address_debug(format!("{id:02x}"))
    }

    pub fn socket_address_debug(&self, stem: impl AsRef<OsStr>) -> PathBuf {
        let mut name = self.shim_name.clone();
        name.push("-");
        name.push(stem.as_ref());
        socket_address(&self.ttrpc_address, name)
    }

    pub async fn event_publisher(&self) -> IoResult<EventPublisher> {
        let publisher = match self.action.as_str() {
            "daemon" => {
                let address = &self.ttrpc_address;
                #[cfg(unix)]
                let address = format!("unix://{address}");
                EventPublisher::connect(address).await?
            }
            _ => EventPublisher::null(),
        };
        let publisher = publisher.with_namespace(&self.namespace);
        Ok(publisher)
    }
}

impl Arguments {
    pub fn parse_env() -> Result<Arguments> {
        Self::parse_from(std::env::args().skip(1), std::env::vars())
    }

    /// Parses command line arguments passed to the shim.
    pub fn parse_from(
        args: impl IntoIterator<Item = impl Into<String>>,
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Result<Arguments> {
        let vars: HashMap<String, String> = vars
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        let mut debug = false;
        let mut version = false;
        let mut id = String::default();
        let mut bundle = PathBuf::default();
        let mut publish_binary = PathBuf::from("containerd");
        let mut namespace = vars
            .get("NAMESPACE")
            .cloned()
            .unwrap_or_else(|| "default".into());
        let mut grpc_address = vars
            .get("GRPC_ADDRESS")
            .cloned()
            .unwrap_or_else(|| CONTAINERD_DEFAULT_ADDRESS.into());

        let args: Vec<String> = args.into_iter().map(|v| v.into()).collect();

        let mut rest: Vec<String> = go_flag::parse_args(&args[..], |f| {
            f.add_flag("debug", &mut debug);
            f.add_flag("v", &mut version);
            f.add_flag("namespace", &mut namespace);
            f.add_flag("id", &mut id);
            f.add_flag("bundle", &mut bundle);
            f.add_flag("address", &mut grpc_address);
            f.add_flag("publish-binary", &mut publish_binary);
        })?;

        let ttrpc_address = vars
            .get("TTRPC_ADDRESS")
            .cloned()
            .unwrap_or_else(|| format!("{grpc_address}.ttrpc"));

        if version {
            return Ok(Arguments {
                action: "version".into(),
                stdout: stdout().duplicate()?.into(),
                ..Default::default()
            });
        }

        ensure!(!rest.is_empty(), "No action specified");

        let mut action = rest.remove(0);

        // If stdout is a terminal, we are running interactively.
        // Skip the daemon launcher step.
        if action == "start" && stdout().is_terminal() {
            action = "daemon".into();
        }

        let shim_name = shim_name();
        let stdout = stdout().duplicate()?.into();

        let args = Arguments {
            id,
            namespace,
            grpc_address,
            ttrpc_address,
            publish_binary,
            debug,
            action,
            rest,
            bundle,
            shim_name,
            stdout,
        };

        match args.action.as_str() {
            "start" | "daemon" | "delete" => Ok(args),
            action => bail!("Unsupported action `{action}`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn parse_all() {
        let args = [
            "-debug",
            "-id",
            "123",
            "-namespace",
            "default",
            "-publish-binary",
            "/path/to/binary",
            "-bundle",
            "bundle",
            "-address",
            "address",
            "delete",
            "abc",
            "def",
        ];

        let envs = [("TTRPC_ADDRESS", "/path/to/c8d.sock")];

        let args = Arguments::parse_from(args, envs).unwrap();

        assert_eq!(args.action, "delete");
        assert!(args.debug);
        assert_eq!(args.id, "123");
        assert_eq!(args.namespace, "default");
        assert_eq!(args.publish_binary, Path::new("/path/to/binary"));
        assert_eq!(args.grpc_address, "address");
        assert_eq!(args.ttrpc_address, "/path/to/c8d.sock");
        assert_eq!(args.bundle, Path::new("bundle"));
    }

    #[test]
    fn parse_flags() {
        let args = ["-id", "123", "-namespace", "default", "start"];
        let args = Arguments::parse_from(args, [] as [(&str, &str); 0]).unwrap();

        assert!(!args.debug);
        assert_eq!(args.id, "123");
        assert_eq!(args.namespace, "default");
    }

    #[test]
    fn parse_version() {
        let args = ["-v"];
        let envs: [(&str, &str); 0] = [];

        let args = Arguments::parse_from(args, envs).unwrap();

        assert_eq!(args.action, "version");
    }

    #[test]
    fn socket_address_with_ext() {
        let args = Arguments {
            ttrpc_address: "/path/to/containerd.sock.ttrpc".into(),
            shim_name: "logger".into(),
            ..Default::default()
        };

        let socket = args.socket_address_debug("123");

        assert_eq!(
            socket,
            PathBuf::from("/path/to/containerd-shim-logger-123.sock.ttrpc")
        );
    }

    #[test]
    fn socket_address_without_ext() {
        let args = Arguments {
            ttrpc_address: "/path/to/containerd-containerd".into(),
            shim_name: "logger".into(),
            ..Default::default()
        };

        let socket = args.socket_address_debug("123");

        assert_eq!(socket, PathBuf::from("/path/to/containerd-shim-logger-123"));
    }
}
