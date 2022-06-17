use crate::{Encoding, PathExt, Result};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// A target to write serialized output to.
#[derive(Debug, Clone, PartialEq)]
pub enum Sink {
    /// Stdout sink.
    Stdout,
    /// Local path sink.
    Path(PathBuf),
}

impl Sink {
    /// Tries to detect the encoding of the sink. Returns `None` if the encoding cannot be
    /// detected.
    pub fn encoding(&self) -> Option<Encoding> {
        match self {
            Self::Stdout => None,
            Self::Path(path) => Encoding::from_path(path),
        }
    }
}

impl From<&str> for Sink {
    fn from(s: &str) -> Self {
        if s == "-" {
            Self::Stdout
        } else {
            Self::Path(PathBuf::from(s))
        }
    }
}

impl From<&Path> for Sink {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_path_buf())
    }
}

impl FromStr for Sink {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(From::from(s))
    }
}

impl fmt::Display for Sink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdout => write!(f, "<stdout>"),
            Self::Path(path) => path
                .relative_to_cwd()
                .unwrap_or_else(|| path.clone())
                .display()
                .fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_str() {
        assert_eq!(Sink::from_str("-"), Ok(Sink::Stdout));
        assert_eq!(
            Sink::from_str("foo.json"),
            Ok(Sink::Path(PathBuf::from("foo.json")))
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(Sink::from("-").encoding(), None);
        assert_eq!(Sink::from("foo").encoding(), None);
        assert_eq!(Sink::from("foo.json").encoding(), Some(Encoding::Json));
    }

    #[test]
    fn test_to_string() {
        assert_eq!(&Sink::Stdout.to_string(), "<stdout>");
        assert_eq!(&Sink::from("Cargo.toml").to_string(), "Cargo.toml");
        assert_eq!(
            &Sink::from(std::fs::canonicalize("src/lib.rs").unwrap().as_path()).to_string(),
            "src/lib.rs"
        );
        assert_eq!(
            &Sink::from("/non-existent/path").to_string(),
            "/non-existent/path"
        );
    }
}
