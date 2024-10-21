use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub mod fd;

pub const DEV_NULL: &str = "/dev/null";

#[cfg(not(target_os = "macos"))]
pub const CONTAINERD_DEFAULT_ADDRESS: &str = r"/run/containerd/containerd.sock";

#[cfg(target_os = "macos")]
pub const CONTAINERD_DEFAULT_ADDRESS: &str = r"/var/run/containerd/containerd.sock";

pub fn socket_address(containerd_socket: impl AsRef<Path>, id: impl AsRef<OsStr>) -> PathBuf {
    let mut name = OsString::from("containerd-shim-");
    name.push(id);
    name.push(".sock");
    containerd_socket.as_ref().with_file_name(name)
}
