//! Utilities to facilitate colorful output.

use crate::paging::{PagingChoice, PagingConfig};
use crate::utils::resolve_cmd;
use bat::{assets::HighlightingAssets, config::Config, controller::Controller, Input, PagingMode};
use clap::ArgEnum;
use dts_core::Encoding;
use once_cell::sync::Lazy;
use std::io::{self, Write};
use std::path::Path;
use termcolor::{ColorSpec, StandardStream, WriteColor};

/// Lazyloaded instance of `HighlightingAssets`. For performance reasons this should only be done
/// once as it's a very heavy operation.
static HIGHLIGHTING_ASSETS: Lazy<HighlightingAssets> = Lazy::new(HighlightingAssets::from_binary);

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
    fn from(cc: ColorChoice) -> Self {
        match cc {
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

/// ColoredStdoutWriter writes data to stdout and may or may not colorize it.
pub struct ColoredStdoutWriter<'a> {
    encoding: Encoding,
    config: HighlightingConfig<'a>,
    buf: Option<Vec<u8>>,
}

impl<'a> ColoredStdoutWriter<'a> {
    /// Creates a new `ColoredStdoutWriter` which colorizes output based on the provided `Encoding`
    /// and `HighlightingConfig`.
    pub fn new(encoding: Encoding, config: HighlightingConfig<'a>) -> Self {
        ColoredStdoutWriter {
            encoding,
            config,
            buf: Some(Vec::with_capacity(256)),
        }
    }

    fn flush_buf(&self, buf: &[u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        let highlighter = SyntaxHighlighter::new(&self.config);

        highlighter.print(self.encoding, buf)
    }
}

impl<'a> io::Write for ColoredStdoutWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.buf.as_mut() {
            Some(w) => w.write(buf),
            None => {
                let mut vec = Vec::with_capacity(256);
                let n = vec.write(buf)?;
                self.buf = Some(vec);
                Ok(n)
            }
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

/// Configuration for the `SyntaxHighlighter`.
#[derive(Default)]
pub struct HighlightingConfig<'a> {
    paging_config: PagingConfig<'a>,
    theme: Option<&'a str>,
}

impl<'a> HighlightingConfig<'a> {
    /// Creates a new `HighlightingConfig`.
    pub fn new(paging_config: PagingConfig<'a>, theme: Option<&'a str>) -> Self {
        HighlightingConfig {
            paging_config,
            theme,
        }
    }

    /// Returns the default theme.
    pub fn default_theme(&self) -> String {
        String::from("base16")
    }

    /// Returns the color theme that should be used to color the output. Checks if the requested
    /// theme is available, otherwise fall back to `base16` as default.
    pub fn theme(&self) -> String {
        self.theme
            .and_then(|requested| {
                let requested = requested.to_lowercase();
                highlighting_assets()
                    .themes()
                    .find(|known| known.to_lowercase() == requested)
                    .map(|theme| theme.to_owned())
            })
            .unwrap_or_else(|| self.default_theme())
    }

    /// Returns a suitable output pager.
    pub fn pager(&self) -> String {
        // Since we are using `bat` to do the syntax highlighting for us we have to ensure that the
        // pager is not `bat` itself. In this case we'll just fall back to using the default pager.
        let pager = self.paging_config.pager();

        if let Some((pager_bin, _)) = resolve_cmd(&pager) {
            if !pager_bin.ends_with("bat") && !pager_bin.ends_with("bat.exe") {
                return pager;
            }
        }

        self.paging_config.default_pager()
    }

    /// Returns the configured `PagingChoice`.
    pub fn paging_choice(&self) -> PagingChoice {
        self.paging_config.paging_choice()
    }
}

/// A syntax highlighter which can highlight a buffer and then print the result to stdout.
pub struct SyntaxHighlighter<'a> {
    config: &'a HighlightingConfig<'a>,
}

impl<'a> SyntaxHighlighter<'a> {
    /// Creates a new `SyntaxHighlighter` with the provided `HighlightingConfig`.
    pub fn new(config: &'a HighlightingConfig<'a>) -> Self {
        SyntaxHighlighter { config }
    }

    /// Hightlights `buf` using the given `Encoding` hint and prints the result to stdout.
    pub fn print(&self, encoding: Encoding, buf: &[u8]) -> io::Result<()> {
        let pager = self.config.pager();

        let config = Config {
            colored_output: true,
            true_color: true,
            pager: Some(&pager),
            paging_mode: self.config.paging_choice().into(),
            theme: self.config.theme(),
            ..Default::default()
        };

        let pseudo_filename = Path::new("out").with_extension(encoding.as_str());
        let input = Input::from_bytes(buf).name(pseudo_filename).into();

        let ctrl = Controller::new(&config, highlighting_assets());

        match ctrl.run(vec![input]) {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }
}

/// Returns the `HighlightingAssets` used for syntax highlighting.
///
/// This is a lazy operation, the assets are only loaded once on the first invocation. Returns a
/// reference to the globally loaded assets thereafter.
pub fn highlighting_assets() -> &'static HighlightingAssets {
    &HIGHLIGHTING_ASSETS
}

/// Prints available themes to stdout.
pub fn print_themes(color_choice: ColorChoice) -> io::Result<()> {
    let example = include_bytes!("assets/example.json");
    let assets = highlighting_assets();

    if color_choice.should_colorize() {
        let max_len = assets.themes().map(str::len).max().unwrap_or(0);

        let mut stdout = StandardStream::stdout(color_choice.into());

        for theme in assets.themes() {
            let config = HighlightingConfig::new(PagingConfig::default(), Some(theme));
            let highlighter = SyntaxHighlighter::new(&config);

            stdout.set_color(ColorSpec::new().set_bold(true))?;
            write!(&mut stdout, "{:1$}", theme, max_len + 2)?;
            stdout.reset()?;

            highlighter.print(Encoding::Json, example)?;
        }
    } else {
        for theme in assets.themes() {
            println!("{}", theme);
        }
    }

    Ok(())
}
