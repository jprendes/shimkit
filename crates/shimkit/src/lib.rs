pub mod args;
pub mod event;
pub mod run;
pub mod utils;

pub use shimkit_types as types;

#[cfg_attr(unix, path = "sys/unix/mod.rs")]
#[cfg_attr(windows, path = "sys/windows/mod.rs")]
mod sys;

mod fs;
mod stdio;

pub use shimkit_macros::main;

pub use trapeze;