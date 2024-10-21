use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub mod fd;

pub const DEV_NULL: &str = "nul";

pub const CONTAINERD_DEFAULT_ADDRESS: &str = r"\\.\pipe\containerd-containerd";

pub fn socket_address(containerd_socket: impl AsRef<Path>, id: impl AsRef<OsStr>) -> PathBuf {
    let mut name = OsString::from("containerd-shim-");
    name.push(id);
    containerd_socket.as_ref().with_file_name(name)
}
