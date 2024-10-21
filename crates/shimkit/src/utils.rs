use std::env::current_exe;
use std::ffi::{OsStr, OsString};
use std::hash::{DefaultHasher, Hash, Hasher as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use oci_spec::runtime::Spec;
use os_str_bytes::OsStrBytesExt as _;
use trapeze::{service, Server, ServerHandle};

use crate::args::Arguments;
use crate::protos::containerd::runtime::sandbox::v1::Sandbox;
use crate::protos::containerd::task::v2::Task;
use crate::sys::socket_address;

pub const GROUP_LABELS: [&str; 2] = [
    "io.kubernetes.cri.sandbox-id",
    "io.containerd.runc.v2.group",
];

pub fn cri_sandbox_id() -> Option<String> {
    if let Ok(spec) = Spec::load("config.json") {
        if let Some(annotations) = spec.annotations() {
            for &label in GROUP_LABELS.iter() {
                if let Some(value) = annotations.get(label) {
                    return Some(value.clone());
                }
            }
        }
    }
    None
}

fn shim_name() -> OsString {
    if let Some(name) = current_exe().unwrap_or_default().file_stem() {
        name.strip_prefix("containerd-shim-")
            .unwrap_or(&name)
            .to_owned()
    } else {
        OsString::from("none")
    }
}

impl Arguments {
    pub fn socket_address(&self, id: impl Hash) -> PathBuf {
        let id = {
            let mut hasher = DefaultHasher::new();
            (&self.namespace, id).hash(&mut hasher);
            hasher.finish()
        };

        self.socker_address_debug(format!("{id:02x}"))
    }

    pub fn socker_address_debug(&self, stem: impl AsRef<OsStr>) -> PathBuf {
        let mut name = shim_name();
        name.push("-");
        name.push(stem.as_ref());
        socket_address(&self.ttrpc_address, name)
    }
}

pub async fn serve(address: impl AsRef<Path>, server: impl Sandbox + Task) -> Result<ServerHandle> {
    let address = address.as_ref().display().to_string();

    #[cfg(unix)]
    let address = format!("unix://{address}");

    let handle = Server::new()
        .register(service!(server : Sandbox + Task))
        .bind(&address)
        .await
        .context("Error binding listener")?;

    Ok(handle)
}
