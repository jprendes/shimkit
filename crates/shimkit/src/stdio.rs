pub use crate::sys::stdio::Duplicate;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{stderr, stdout, Read, Seek, Write};

    use super::*;

    macro_rules! impl_std_guard {
        ($StdxxxGuard:ident, $stdxxx:ident, $duplicate_to_stdxxx:ident) => {
            struct $StdxxxGuard(File);
            impl $StdxxxGuard {
                pub fn new() -> Self {
                    Self(File::from($stdxxx().duplicate().unwrap()))
                }
            }
            impl Drop for $StdxxxGuard {
                fn drop(&mut self) {
                    self.0.$duplicate_to_stdxxx().unwrap();
                }
            }
        };
    }

    impl_std_guard!(StdoutGuard, stdout, duplicate_to_stdout);
    impl_std_guard!(StderrGuard, stderr, duplicate_to_stderr);

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
            sink.duplicate_to_stdout().unwrap();
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
            sink.duplicate_to_stderr().unwrap();
            // use writeln! instad of eprintln! as eprintln! gets captured in tests
            let _ = writeln!(stderr(), "hello world!");
        }
        let buf = read_all_to_string(sink);
        assert_eq!(buf, "hello world!\n");
    }
}
