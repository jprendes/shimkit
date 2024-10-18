use std::os::windows::prelude::{AsRawHandle, FromRawHandle};

use libc::{get_osfhandle, intptr_t, open_osfhandle, O_APPEND};

pub type RawFd = libc::c_int;

pub const STDOUT_FILENO: RawFd = 1;
pub const STDERR_FILENO: RawFd = 2;

pub trait AsRawFd {
    fn as_raw_fd(&self) -> RawFd;
}

pub trait FromRawFd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self;
}

impl<H: AsRawHandle> AsRawFd for H {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { open_osfhandle(self.as_raw_handle() as intptr_t, O_APPEND) }
    }
}

impl<H: FromRawHandle> FromRawFd for H {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let handle = get_osfhandle(fd);
        Self::from_raw_handle(handle as *mut _)
    }
}
