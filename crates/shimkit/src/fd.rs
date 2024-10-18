use std::fs::File;

use crate::sys::fd::{AsRawFd, FromRawFd as _, STDERR_FILENO, STDOUT_FILENO};

pub trait FdRedirect: AsRawFd {
    fn use_as_stdout(&self) {
        let fd = self.as_raw_fd();
        unsafe { libc::dup2(fd, STDOUT_FILENO) };
    }

    fn use_as_stderr(&self) {
        let fd = self.as_raw_fd();
        unsafe { libc::dup2(fd, STDERR_FILENO) };
    }
}

impl<F: AsRawFd> FdRedirect for F {}

pub fn clone_stdout() -> File {
    unsafe { File::from_raw_fd(libc::dup(STDOUT_FILENO)) }
}

pub fn clone_stderr() -> File {
    unsafe { File::from_raw_fd(libc::dup(STDERR_FILENO)) }
}
