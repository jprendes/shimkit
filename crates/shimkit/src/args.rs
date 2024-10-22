use std::collections::HashMap;
use std::env::current_exe;
use std::ffi::{OsStr, OsString};
use std::hash::{DefaultHasher, Hash, Hasher as _};
use std::io::{stdout, IsTerminal};
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Result};
use os_str_bytes::OsStrBytesExt as _;

use crate::run::AddressPipe;
use crate::sys::CONTAINERD_DEFAULT_ADDRESS;

#[derive(Debug)]
pub enum Command {
    Start { pipe: AddressPipe, args: Arguments },
    Delete { bundle: PathBuf, args: Arguments },
    Version,
}

#[derive(Default, Clone)]
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

    pub(crate) is_daemon: bool,
    pub(crate) rest: Vec<String>,
    pub(crate) shim_name: OsString,
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

impl Arguments {
    pub(crate) fn to_args_vec(&self, command: &'static OsStr) -> Vec<&OsStr> {
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
        args.push(command.as_ref());
        args.extend(self.rest.iter().map(AsRef::<OsStr>::as_ref));
        args
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
}

impl Command {
    pub fn parse_env() -> Result<Command> {
        Self::parse_from(std::env::args().skip(1), std::env::vars())
    }

    /// Parses command line arguments passed to the shim.
    pub fn parse_from(
        args: impl IntoIterator<Item = impl Into<String>>,
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Result<Command> {
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
            return Ok(Command::Version);
        }

        ensure!(!rest.is_empty(), "No action specified");

        let action = rest.remove(0);

        // If stdout is a terminal, we are running interactively.
        // Skip the daemon launcher step.
        let is_daemon = action == "daemon" || stdout().is_terminal();

        let shim_name = shim_name();

        let args = Arguments {
            id,
            namespace,
            grpc_address,
            ttrpc_address,
            publish_binary,
            debug,
            is_daemon,
            rest,
            shim_name,
        };

        let action = match action.as_str() {
            "start" | "daemon" => Command::Start {
                pipe: AddressPipe::from_stdout(),
                args,
            },
            "delete" => Command::Delete { bundle, args },
            _ => bail!("Unsupported action `{action}`"),
        };

        Ok(action)
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

        let command = Command::parse_from(args, envs).unwrap();

        let Command::Delete { bundle, args } = command else {
            panic!("Wrong action");
        };
        assert!(args.debug);
        assert_eq!(args.id, "123");
        assert_eq!(args.namespace, "default");
        assert_eq!(args.publish_binary, Path::new("/path/to/binary"));
        assert_eq!(args.grpc_address, "address");
        assert_eq!(args.ttrpc_address, "/path/to/c8d.sock");
        assert_eq!(bundle, Path::new("bundle"));
    }

    #[test]
    fn parse_flags() {
        let args = ["-id", "123", "-namespace", "default", "start"];
        let command = Command::parse_from(args, [] as [(&str, &str); 0]).unwrap();

        let Command::Start { args, .. } = command else {
            panic!("Wrong action");
        };

        assert!(!args.debug);
        assert_eq!(args.id, "123");
        assert_eq!(args.namespace, "default");
    }

    #[test]
    fn parse_version() {
        let args = ["-v"];
        let envs: [(&str, &str); 0] = [];

        let action = Command::parse_from(args, envs).unwrap();

        let Command::Version = action else {
            panic!("Wrong action");
        };
    }

    #[test]
    fn socket_address_with_ext() {
        let args = Arguments {
            ttrpc_address: "/path/to/containerd.socket.ttrpc".into(),
            shim_name: "logger".into(),
            ..Default::default()
        };

        let socket = args.socket_address_debug("123");

        assert_eq!(
            socket,
            PathBuf::from("/path/to/containerd-shim-logger-123.socket.ttrpc")
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

        assert_eq!(
            socket,
            PathBuf::from("/path/to/containerd-shim-logger-123")
        );
    }
}
