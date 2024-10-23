use std::fs::File;
use std::io::Result;
use std::path::Path;

use crate::sys::DEV_NULL;

pub trait FileEx: Sized {
    fn append(path: impl AsRef<Path>) -> Result<Self>;
}

impl FileEx for File {
    fn append(path: impl AsRef<Path>) -> Result<Self> {
        File::options()
            .create(false)
            .read(false)
            .append(true)
            .open(path)
    }
}

pub fn dev_null() -> Result<File> {
    File::append(DEV_NULL)
}
