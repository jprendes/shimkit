use std::io::{Error, Result};
use std::os::windows::io::{AsHandle, AsRawHandle, IntoRawHandle, OwnedHandle};

use libc::{c_int, close, open_osfhandle, O_APPEND};

type RawFd = c_int;

const STDOUT_FILENO: RawFd = 1;
const STDERR_FILENO: RawFd = 2;

pub trait Duplicate: AsHandle {
    fn duplicate(&self) -> Result<OwnedHandle> {
        self.as_handle().try_clone_to_owned()
    }

    unsafe fn duplicate_to_fd(&self, dst: RawFd) -> Result<()> {
        let owned = self.duplicate()?;
        let fd = into_fd(owned)?;
        let new = unsafe { libc::dup2(fd.0, dst) };
        if new < 0 {
            return Err(Error::other("Failed to clone file descriptor"));
        }
        Ok(())
    }

    fn duplicate_to_stdout(&self) -> Result<()> {
        unsafe { self.duplicate_to_fd(STDOUT_FILENO) }
    }

    fn duplicate_to_stderr(&self) -> Result<()> {
        unsafe { self.duplicate_to_fd(STDERR_FILENO) }
    }
}

impl<F: AsHandle> Duplicate for F {}

struct OwnedFd(RawFd);

impl Drop for OwnedFd {
    fn drop(&mut self) {
        unsafe { close(self.0) };
    }
}

fn into_fd(handle: OwnedHandle) -> Result<OwnedFd> {
    let fd = unsafe { open_osfhandle(handle.as_raw_handle() as _, O_APPEND) };
    if fd < 0 {
        return Err(Error::other("Failed to get file descriptor"));
    }
    let _ = handle.into_raw_handle(); // drop ownership of the handle, it's managed by fd now
    Ok(OwnedFd(fd))
}
