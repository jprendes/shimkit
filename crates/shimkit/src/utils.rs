use std::path::Path;

use anyhow::{Context, Result};
use oci_spec::runtime::Spec;
use trapeze::{service, Server, ServerHandle};

use crate::protos::containerd::runtime::sandbox::v1::Sandbox;
use crate::protos::containerd::task::v2::Task;

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
