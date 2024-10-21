use std::env::{current_dir, current_exe};
use std::fs::File;
use std::io::{copy, stderr, stdout, IsTerminal as _, Result as IoResult, Write as _};
use std::path::Path;
use std::process::{exit, Command as ProcessCmd, Stdio, Termination};

use anyhow::Context;

use crate::args::Command;
use crate::fd::{clone_stderr, clone_stdout, FdRedirect as _};
use crate::fs::{open_append, open_dev_null};

#[derive(Debug)]
pub struct AddressPipe(File);

impl AddressPipe {
    pub(crate) fn from_file(file: File) -> Self {
        Self(file)
    }

    pub(crate) fn from_stdout() -> Self {
        Self::from_file(clone_stdout())
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

/// Shim entry point that must be invoked from `main`.
pub fn run<T: Termination>(f: impl FnOnce(Command) -> T) -> anyhow::Result<T> {
    let action = Command::parse_env()?;

    if let Command::Start { args, .. } = &action {
        if !args.is_daemon {
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
            // Redirect stdout and stderr to the logs file.
            let log = if let Ok(file) = open_append("log") {
                file
            } else if stderr().is_terminal() {
                clone_stderr()
            } else {
                open_dev_null().context("failed to allocate a sink for stdout")?
            };

            log.use_as_stdout();
            log.use_as_stderr();
        }
    }

    Ok(f(action))
}
