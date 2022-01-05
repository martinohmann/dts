//! Contains an `io::Write` implementation that is capable to pipe output through a pager and
//! utilities to decide if output should be colored or not.

use crate::{
    paging::{PagingChoice, PagingConfig},
    utils::resolve_cmd,
};
use clap::ArgEnum;
use std::io::{self, Stdout, Write};
use std::process::{Child, Command, Stdio};
use termcolor::{Buffer, BufferWriter, ColorSpec, WriteColor};

/// ColorChoice represents the color preference of a user.
#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
pub enum ColorChoice {
    /// Always color output even if stdout is a file.
    Always,
    /// Automatically detect if output should be colored. Coloring is disabled if stdout is not
    /// interactive or a dumb term, or the user explicitly disabled colors via `NO_COLOR`
    /// environment variable.
    Auto,
    /// Never color output.
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        ColorChoice::Never
    }
}

impl From<ColorChoice> for termcolor::ColorChoice {
    fn from(choice: ColorChoice) -> Self {
        match choice {
            ColorChoice::Always => termcolor::ColorChoice::Always,
            ColorChoice::Auto => termcolor::ColorChoice::Auto,
            ColorChoice::Never => termcolor::ColorChoice::Never,
        }
    }
}

impl ColorChoice {
    /// Returns true if the `ColorChoice` indicates that coloring is enabled.
    pub fn should_colorize(&self) -> bool {
        match *self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => self.env_allows_color() && atty::is(atty::Stream::Stdout),
        }
    }

    #[cfg(not(windows))]
    fn env_allows_color(&self) -> bool {
        match std::env::var_os("TERM") {
            None => return false,
            Some(k) => {
                if k == "dumb" {
                    return false;
                }
            }
        }

        std::env::var_os("NO_COLOR").is_none()
    }

    #[cfg(windows)]
    fn env_allows_color(&self) -> bool {
        // On Windows, if TERM isn't set, then we shouldn't automatically
        // assume that colors aren't allowed. This is unlike Unix environments
        // where TERM is more rigorously set.
        if let Some(k) = std::env::var_os("TERM") {
            if k == "dumb" {
                return false;
            }
        }

        std::env::var_os("NO_COLOR").is_none()
    }
}

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
        match resolve_cmd(config.pager()) {
            Some((pager_bin, args)) => {
                let mut cmd = Command::new(&pager_bin);

                if pager_bin.ends_with("less") || pager_bin.ends_with("less.exe") {
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

/// A type that can buffer writes of colored and non-colored data and finally print the buffer to
/// stdout.
pub struct BufferedStdoutPrinter {
    stdout: BufferWriter,
    buf: Buffer,
}

impl BufferedStdoutPrinter {
    /// Creates a new `BufferedStdoutPrinter` which may colorize output based on the provided
    /// `ColorChoice`.
    pub fn new(choice: ColorChoice) -> Self {
        let stdout = BufferWriter::stdout(choice.into());

        BufferedStdoutPrinter {
            buf: stdout.buffer(),
            stdout,
        }
    }

    /// Writes `buf` wrapped with the given `ColorSpec` into the print buffer.
    ///
    /// The `ColorSpec` may be discarded if the `ColorChoice` used for creating the
    /// `BufferedStdoutPrinter` dictated that output should not be colored.
    pub fn write_colored<B>(&mut self, spec: &ColorSpec, buf: B) -> io::Result<()>
    where
        B: AsRef<[u8]>,
    {
        self.buf.set_color(spec)?;
        self.write(buf)?;
        self.buf.reset()
    }

    /// Writes `buf` into the internal print buffer.
    pub fn write<B>(&mut self, buf: B) -> io::Result<()>
    where
        B: AsRef<[u8]>,
    {
        self.buf.write_all(buf.as_ref())
    }

    /// Prints the contents of the print buffer to stdout.
    pub fn print(&self) -> io::Result<()> {
        self.stdout.print(&self.buf)
    }
}
