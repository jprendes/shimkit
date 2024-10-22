pub mod fd;

pub const DEV_NULL: &str = "/dev/null";

#[cfg(not(target_os = "macos"))]
pub const CONTAINERD_DEFAULT_ADDRESS: &str = r"/run/containerd/containerd.sock";

#[cfg(target_os = "macos")]
pub const CONTAINERD_DEFAULT_ADDRESS: &str = r"/var/run/containerd/containerd.sock";
