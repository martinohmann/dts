//! Utilities to facilitate colorful output.

use bat::{assets::HighlightingAssets, Input, PrettyPrinter};
use clap::ArgEnum;
use dts_core::Encoding;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

/// Returns the names of the themes available for syntax highlighting.
pub fn themes() -> Vec<String> {
    HighlightingAssets::from_binary()
        .themes()
        .map(|theme| theme.to_string())
        .collect()
}

/// StdoutWriter writes data to stdout and may or may not colorize it.
pub struct StdoutWriter<'a> {
    color_choice: ColorChoice,
    encoding: Encoding,
    theme: Option<&'a str>,
    buf: Option<Vec<u8>>,
}

impl<'a> StdoutWriter<'a> {
    /// Creates a new `StdoutWriter` which may colorize output using the provided theme and
    /// `Encoding` hint based on the `ColorChoice`.
    pub fn new(color_choice: ColorChoice, encoding: Encoding, theme: Option<&'a str>) -> Self {
        StdoutWriter {
            color_choice,
            encoding,
            theme,
            buf: Some(Vec::with_capacity(256)),
        }
    }

    // The pseudo filename will determine the syntax highlighting used by the PrettyPrinter.
    fn pseudo_filename(&self) -> PathBuf {
        Path::new("out").with_extension(self.encoding.as_str())
    }

    // Checks if the `PrettyPrinter` knows the requested theme, otherwise fall back to `base16` as
    // default.
    fn theme(&self, printer: &PrettyPrinter) -> &str {
        self.theme
            .and_then(|requested| {
                printer
                    .themes()
                    .find(|known| known == &requested)
                    .and(Some(requested))
            })
            .unwrap_or("base16")
    }

    fn should_colorize(&self, buf: &[u8]) -> bool {
        match self.color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                // Only highlight if the buffer is <= 1MB by default.
                // Syntax highlighting of multiple thousand lines is slow.
                self.color_choice.should_colorize() && buf.len() <= 1_048_576
            }
        }
    }

    fn flush_buf(&self, buf: &[u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        if !self.should_colorize(buf) {
            return io::stdout().write_all(buf);
        }

        let mut printer = PrettyPrinter::new();
        let theme = self.theme(&printer);
        let input = Input::from_bytes(buf).name(self.pseudo_filename());

        match printer.input(input).theme(theme).print() {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }
}

impl<'a> io::Write for StdoutWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.buf.as_mut() {
            Some(w) => w.write(buf),
            None => panic!("StdoutWriter was already flushed"),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.buf.take() {
            Some(buf) => self.flush_buf(&buf),
            None => Ok(()),
        }
    }
}

impl<'a> Drop for StdoutWriter<'a> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
