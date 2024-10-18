use std::fs::File;
use std::io::Result;
use std::path::Path;

use crate::sys::DEV_NULL;

pub fn open_append(path: impl AsRef<Path>) -> Result<File> {
    File::options()
        .create(false)
        .read(false)
        .append(true)
        .open(path)
}

pub fn open_dev_null() -> Result<File> {
    open_append(DEV_NULL)
}
