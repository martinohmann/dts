//! Utilities to facilitate output paging.

use clap::ArgEnum;

/// PagingChoice represents the paging preference of a user.
#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
pub enum PagingChoice {
    /// Always page output even if you would fit on the screen.
    Always,
    /// Automatically decide when to page output. This will page output only if it does not fit on
    /// the screen.
    Auto,
    /// Never page output.
    Never,
}

/// PagingConfig holds configuration related to output paging.
pub struct PagingConfig<'a> {
    choice: PagingChoice,
    pager: Option<&'a str>,
}

impl<'a> PagingConfig<'a> {
    /// Creates a new `PagingConfig` with given `PagingChoice` and an optional pager.
    pub fn new(choice: PagingChoice, pager: Option<&'a str>) -> Self {
        PagingConfig { choice, pager }
    }

    /// Returns the default pager command.
    pub fn default_pager(&self) -> String {
        String::from("less")
    }

    /// Returns a suitable output pager. This will either use a) the explicitly configured pager b)
    /// the contents of the `PAGER` environment variable or c) the default pager which is `less`.
    pub fn pager(&self) -> String {
        match self.pager {
            Some(cmd) if !cmd.is_empty() => cmd.to_owned(),
            _ => match std::env::var("PAGER").ok() {
                Some(cmd) if !cmd.is_empty() => cmd,
                _ => self.default_pager(),
            },
        }
    }

    /// Returns a reference to the configured `PagingChoice`.
    pub fn paging_choice(&self) -> PagingChoice {
        self.choice
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pager() {
        let config = PagingConfig::new(PagingChoice::Auto, Some("my-pager"));
        assert_eq!(&config.pager(), "my-pager");

        let config = PagingConfig::new(PagingChoice::Auto, None);
        assert_eq!(&config.pager(), &config.default_pager());

        let config = PagingConfig::new(PagingChoice::Auto, Some(""));
        assert_eq!(&config.pager(), &config.default_pager());

        let config = PagingConfig::new(PagingChoice::Auto, None);
        temp_env::with_var("PAGER", Some("more"), || {
            assert_eq!(&config.pager(), "more")
        });

        temp_env::with_var("PAGER", Some(""), || {
            assert_eq!(&config.pager(), &config.default_pager())
        });

        temp_env::with_var("PAGER", None::<&str>, || {
            assert_eq!(&config.pager(), &config.default_pager())
        });
    }
}