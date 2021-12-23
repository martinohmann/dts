use anyhow::{anyhow, Result};
use bat::{assets::HighlightingAssets, Input, PrettyPrinter};
use clap::ArgEnum;
use dts_core::Encoding;
use std::{env, fmt, path::Path};

/// ColorChoice represents the color preference of a user.
#[derive(ArgEnum, Debug, PartialEq, Clone)]
pub enum ColorChoice {
    /// Always color output even if stdout is a file.
    Always,
    /// Automatically detect if output should be colored. Coloring is disabled if stdout is not
    /// interactive or a dumb term, or the user explicitly disabled colors via `NO_COLOR`
    /// environment variable.
    Auto,
    /// Never color output.
    #[clap(alias = "off")]
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
        match env::var_os("TERM") {
            None => return false,
            Some(k) => {
                if k == "dumb" {
                    return false;
                }
            }
        }

        env::var_os("NO_COLOR").is_none()
    }

    #[cfg(windows)]
    fn env_allows_color(&self) -> bool {
        // On Windows, if TERM isn't set, then we shouldn't automatically
        // assume that colors aren't allowed. This is unlike Unix environments
        // where TERM is more rigorously set.
        if let Some(k) = env::var_os("TERM") {
            if k == "dumb" {
                return false;
            }
        }

        env::var_os("NO_COLOR").is_none()
    }
}

impl fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorChoice::Always => f.write_str("always"),
            ColorChoice::Auto => f.write_str("auto"),
            ColorChoice::Never => f.write_str("never"),
        }
    }
}

/// Returns the names of the themes available for syntax highlighting.
pub fn themes() -> Vec<String> {
    HighlightingAssets::from_binary()
        .themes()
        .map(|theme| theme.to_string())
        .collect()
}

/// Applies syntax highlighting to the contents of `buf` based on the `Encoding` and theme and
/// prints the result to stdout.
pub fn print_highlighted(buf: &[u8], encoding: Encoding, theme: Option<&str>) -> Result<()> {
    // The pseudo filename will determine the syntax highlighting used by the PrettyPrinter.
    let filename = Path::new("out").with_extension(encoding.as_str());
    let input = Input::from_bytes(buf).name(filename);

    let mut printer = PrettyPrinter::new();

    // Check if the printer knows the requested theme, otherwise fall back to the `base16` as
    // default.
    let theme = theme
        .and_then(|requested| {
            printer
                .themes()
                .find(|known| known == &requested)
                .and(Some(requested))
        })
        .unwrap_or("base16");

    printer
        .input(input)
        .theme(theme)
        .print()
        .map(|_| ())
        .map_err(|err| anyhow!("{}", err))
}
