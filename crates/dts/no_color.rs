//! Contains an `io::Write` implementation that is capable to pipe output through a pager.

use crate::paging::{PagingChoice, PagingConfig};
use std::io::{self, Stdout};
use std::process::{Child, Command, Stdio};

/// StdoutWriter either writes data directly to stdout or passes it through a pager first.
#[derive(Debug)]
pub enum StdoutWriter {
    Pager(Child),
    Stdout(Stdout),
}

impl StdoutWriter {
    /// Creates a new `StdoutWriter` which may page output based on the `PagingConfig`.
    pub fn new(config: PagingConfig<'_>) -> Self {
        match config.paging_choice() {
            PagingChoice::Always | PagingChoice::Auto => StdoutWriter::pager(config),
            PagingChoice::Never => StdoutWriter::stdout(),
        }
    }

    /// Tries to launch the pager. Falls back to `io::Stdout` in case of errors.
    fn pager(config: PagingConfig<'_>) -> Self {
        match shell_words::split(&config.pager()) {
            Err(_) => StdoutWriter::stdout(),
            Ok(parts) => match parts.split_first() {
                Some((pager, args)) => {
                    let mut cmd = Command::new(&pager);

                    if pager == "less" {
                        if args.is_empty() {
                            if let PagingChoice::Auto = config.paging_choice() {
                                cmd.arg("--quit-if-one-screen");
                            }
                        } else {
                            cmd.args(args);
                        }

                        cmd.env("LESSCHARSET", "UTF-8");
                    } else {
                        cmd.args(args);
                    }

                    cmd.stdin(Stdio::piped())
                        .spawn()
                        .map(StdoutWriter::Pager)
                        .unwrap_or_else(|_| StdoutWriter::stdout())
                }
                None => StdoutWriter::stdout(),
            },
        }
    }

    fn stdout() -> Self {
        StdoutWriter::Stdout(io::stdout())
    }
}

impl io::Write for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            StdoutWriter::Pager(Child { stdin, .. }) => stdin.as_mut().unwrap().write(buf),
            StdoutWriter::Stdout(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            StdoutWriter::Pager(Child { stdin, .. }) => stdin.as_mut().unwrap().flush(),
            StdoutWriter::Stdout(stdout) => stdout.flush(),
        }
    }
}

impl Drop for StdoutWriter {
    fn drop(&mut self) {
        if let StdoutWriter::Pager(cmd) = self {
            let _ = cmd.wait();
        }
    }
}
