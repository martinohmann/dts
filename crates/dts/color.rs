//! Utilities to facilitate colorful output.

use crate::paging::{PagingChoice, PagingConfig};
use bat::{assets::HighlightingAssets, Input, PagingMode, PrettyPrinter};
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

/// ColoredStdoutWriter writes data to stdout and may or may not colorize it.
pub struct ColoredStdoutWriter<'a> {
    encoding: Encoding,
    theme: Option<&'a str>,
    buf: Option<Vec<u8>>,
    paging_config: PagingConfig<'a>,
}

impl<'a> ColoredStdoutWriter<'a> {
    /// Creates a new `ColoredStdoutWriter` which may colorize output using the provided `Encoding` hint
    /// based on the `ColorConfig`.
    pub fn new(
        encoding: Encoding,
        theme: Option<&'a str>,
        paging_config: PagingConfig<'a>,
    ) -> Self {
        ColoredStdoutWriter {
            encoding,
            theme,
            buf: Some(Vec::with_capacity(256)),
            paging_config,
        }
    }

    // The pseudo filename will determine the syntax highlighting used by the PrettyPrinter.
    fn pseudo_filename(&self) -> PathBuf {
        Path::new("out").with_extension(self.encoding.as_str())
    }

    // Returns the color theme that should be used to color the output. Checks if the
    // `PrettyPrinter` knows the requested theme, otherwise fall back to `base16` as default.
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

    // Returns a suitable output pager. Since we are using the `bat` pretty printer we have to
    // ensure that the pager is not `bat` itself. In this case we'll just fall back to using the
    // default pager.
    fn pager(&self) -> String {
        let pager = self.paging_config.pager();

        if let Ok(parts) = shell_words::split(&pager) {
            if let Some((cmd, _)) = parts.split_first() {
                if !Path::new(cmd).ends_with("bat") {
                    return pager;
                }
            }
        }

        self.paging_config.default_pager()
    }

    fn paging_mode(&self) -> PagingMode {
        self.paging_config.paging_choice().into()
    }

    fn flush_buf(&self, buf: &[u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        let mut printer = PrettyPrinter::new();

        let theme = self.theme(&printer);
        let input = Input::from_bytes(buf).name(self.pseudo_filename());

        match printer
            .paging_mode(self.paging_mode())
            .pager(&self.pager())
            .input(input)
            .theme(theme)
            .print()
        {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }
}

impl<'a> io::Write for ColoredStdoutWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.buf.as_mut() {
            Some(w) => w.write(buf),
            None => panic!("ColoredStdoutWriter was already flushed"),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.buf.take() {
            Some(buf) => self.flush_buf(&buf),
            None => Ok(()),
        }
    }
}

impl<'a> Drop for ColoredStdoutWriter<'a> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl From<PagingChoice> for PagingMode {
    fn from(choice: PagingChoice) -> Self {
        match choice {
            PagingChoice::Always => PagingMode::Always,
            PagingChoice::Auto => PagingMode::QuitIfOneScreen,
            PagingChoice::Never => PagingMode::Never,
        }
    }
}
