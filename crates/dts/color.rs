use clap::ArgEnum;
use std::env;
use std::fmt;

// Re-exports
pub use bat::{assets::HighlightingAssets, Input, PrettyPrinter};

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
