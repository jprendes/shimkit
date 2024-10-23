use std::io::{Error, Result};
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};

use libc::{STDERR_FILENO, STDOUT_FILENO};

pub trait Duplicate: AsFd {
    fn duplicate(&self) -> Result<OwnedFd> {
        self.as_fd().try_clone_to_owned()
    }

    unsafe fn duplicate_to_fd(&self, dst: RawFd) -> Result<()> {
        let new = unsafe { libc::dup2(self.as_fd().as_raw_fd(), dst) };
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

impl<F: AsFd> Duplicate for F {}
