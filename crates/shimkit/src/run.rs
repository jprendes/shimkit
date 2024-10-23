use std::env::{current_dir, current_exe};
use std::fs::File;
use std::io::{copy, stderr, stdout, IsTerminal as _, Result as IoResult, Write as _};
use std::path::Path;
use std::process::{exit, Command as ProcessCmd, Stdio, Termination};

use anyhow::Context;

use crate::args::Command;
use crate::fs::{dev_null, FileEx as _};
use crate::stdio::Duplicate as _;

#[derive(Debug)]
pub struct AddressPipe(File);

impl AddressPipe {
    pub(crate) fn from_file(file: File) -> Self {
        Self(file)
    }

    pub(crate) fn from_stdout() -> IoResult<Self> {
        Ok(Self::from_file(stdout().duplicate()?.into()))
    }

    pub fn write_address(mut self, address: impl AsRef<Path>) -> IoResult<()> {
        #[cfg(unix)]
        write!(self.0, "unix://")?;
        writeln!(self.0, "{}", address.as_ref().display())?;
        Ok(())
    }

    pub fn is_terminal(&self) -> bool {
        self.0.is_terminal()
    }
}

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
pub fn run<T: Termination>(f: impl FnOnce(Command) -> T) -> anyhow::Result<T> {
    let action = Command::parse_env()?;

    if let Command::Start { args, .. } = &action {
        if !args.is_daemon {
            // This is not the daemon, but rather it's the daemon launcher
            // Re-spawn itself as a daemon

            let cmd = current_exe()?;
            let cwd = current_dir()?;

            let mut child = ProcessCmd::new(cmd)
                .current_dir(cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .args(args.to_args_vec("daemon".as_ref()))
                .spawn()?;

            // safe, since we piped stdout
            let mut output = child.stdout.take().unwrap();

            copy(&mut output, &mut stdout())?;
            stdout().flush()?;

            exit(0);
        } else {
            // This is the daemon

            // Before handing over control to user code, redirect stdout/stderr
            let log = open_log().context("failed to allocate a sink for stdout")?;
            log.duplicate_to_stdout()?;
            log.duplicate_to_stderr()?;
        }
    }

    Ok(f(action))
}
