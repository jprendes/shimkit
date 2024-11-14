use std::env::{current_dir, current_exe};
use std::fs::File;
use std::io::{copy, stderr, stdout, IsTerminal as _, Result as IoResult, Write as _};
use std::process::{exit, Command, Stdio, Termination};

use anyhow::Context;

use crate::args::Arguments;
use crate::fs::{dev_null, FileEx as _};
use crate::stdio::Duplicate as _;

fn open_log() -> IoResult<File> {
    // try with the log file
    if let Ok(file) = File::append("log") {
        return Ok(file);
    }
    // if we are running interactively, try with stderr
    if stderr().is_terminal() {
        if let Ok(file) = stderr().duplicate() {
            return Ok(file.into());
        }
    }
    // try with a null sink
    dev_null()
}

/// Shim entry point that must be invoked from `main`.
pub fn run<T: Termination>(f: impl FnOnce(Arguments) -> T) -> anyhow::Result<T> {
    let arguments = Arguments::parse_env()?;

    match arguments.action.as_str() {
        "start" => {
            // This is the daemon launcher, re-spawn itself as a daemon
            let cmd = current_exe()?;
            let cwd = current_dir()?;

            let mut child = Command::new(cmd)
                .current_dir(cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .args(arguments.to_args_vec("daemon".as_ref()))
                .spawn()?;

            // safe, since we piped stdout
            let mut output = child.stdout.take().unwrap();

            copy(&mut output, &mut stdout())?;
            stdout().flush()?;

            exit(0);
        }
        _ => {
            // This is the daemon

            // Before handing over control to user code, redirect stdout/stderr
            let log = open_log().context("failed to allocate a sink for stdout")?;
            log.duplicate_to_stdout()?;
            log.duplicate_to_stderr()?;

            Ok(f(arguments))
        }
    }
}
