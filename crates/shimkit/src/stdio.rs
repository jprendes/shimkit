use crate::sys::fd::{AsRawFd, FromRawFd, STDERR_FILENO, STDOUT_FILENO};

pub trait FdRedirect: AsRawFd + FromRawFd {
    fn use_as_stdout(&self) {
        let fd = self.as_raw_fd();
        unsafe { libc::dup2(fd, STDOUT_FILENO) };
    }

    fn use_as_stderr(&self) {
        let fd = self.as_raw_fd();
        unsafe { libc::dup2(fd, STDERR_FILENO) };
    }

    fn stdout() -> Self
    where
        Self: Sized,
    {
        unsafe { Self::from_raw_fd(libc::dup(STDOUT_FILENO)) }
    }

    fn stderr() -> Self
    where
        Self: Sized,
    {
        unsafe { Self::from_raw_fd(libc::dup(STDERR_FILENO)) }
    }
}

impl<F: AsRawFd + FromRawFd> FdRedirect for F {}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{stderr, stdout, Read, Seek, Write};

    use super::*;

    macro_rules! impl_std_guard {
        ($StdxxxGuard:ident, $stdxxx:ident, $use_as_stdxxx:ident) => {
            struct $StdxxxGuard(File);
            impl $StdxxxGuard {
                pub fn new() -> Self {
                    Self(File::$stdxxx())
                }
            }
            impl Drop for $StdxxxGuard {
                fn drop(&mut self) {
                    self.0.$use_as_stdxxx();
                }
            }
        };
    }

    impl_std_guard!(StdoutGuard, stdout, use_as_stdout);
    impl_std_guard!(StderrGuard, stderr, use_as_stderr);

    fn read_all_to_string(mut f: File) -> String {
        let mut buf = String::new();
        f.seek(std::io::SeekFrom::Start(0)).unwrap();
        f.read_to_string(&mut buf).unwrap();
        buf
    }

    #[test]
    fn redirection_stdout() {
        let _lock = stdout().lock();
        let sink = tempfile::tempfile().unwrap();
        {
            let _guard = StdoutGuard::new();
            sink.use_as_stdout();
            // use writeln! instad of println! as println! gets captured in tests
            let _ = writeln!(stdout(), "hello world!");
        }
        let buf = read_all_to_string(sink);
        assert_eq!(buf, "hello world!\n");
    }

    #[test]
    fn redirection_stderr() {
        let _lock = stderr().lock();
        let sink = tempfile::tempfile().unwrap();
        {
            let _guard = StderrGuard::new();
            sink.use_as_stderr();
            // use writeln! instad of eprintln! as eprintln! gets captured in tests
            let _ = writeln!(stderr(), "hello world!");
        }
        let buf = read_all_to_string(sink);
        assert_eq!(buf, "hello world!\n");
    }

    #[test]
    fn cloning_stdout() {
        let _lock = stdout().lock();
        let sink = tempfile::tempfile().unwrap();
        {
            let _guard = StdoutGuard::new();
            sink.use_as_stdout();
            let mut f = File::stdout();
            assert_ne!(f.as_raw_fd(), STDOUT_FILENO);
            assert_ne!(f.as_raw_fd(), sink.as_raw_fd());
            let _ = writeln!(f, "hello world!");
        }
        let buf = read_all_to_string(sink);
        assert_eq!(buf, "hello world!\n");
    }

    #[test]
    fn cloning_stderr() {
        let _lock = stderr().lock();
        let sink = tempfile::tempfile().unwrap();
        {
            let _guard = StderrGuard::new();
            sink.use_as_stderr();
            let mut f = File::stderr();
            assert_ne!(f.as_raw_fd(), STDERR_FILENO);
            assert_ne!(f.as_raw_fd(), sink.as_raw_fd());
            let _ = writeln!(f, "hello world!");
        }
        let buf = read_all_to_string(sink);
        assert_eq!(buf, "hello world!\n");
    }
}
