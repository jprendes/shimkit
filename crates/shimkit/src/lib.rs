pub mod args;
pub mod protos;
pub mod run;
pub mod utils;

#[cfg_attr(unix, path = "sys/unix/mod.rs")]
#[cfg_attr(windows, path = "sys/windows/mod.rs")]
mod sys;

mod fd;
mod fs;

pub use shimkit_macros::main;
