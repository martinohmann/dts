use std::fmt;
use thiserror::Error;

/// Error emitted by all parsers in this module.
#[derive(Error, Debug)]
#[error("Failed to parse {kind}:\n{msg}")]
pub struct ParseError {
    kind: ParseErrorKind,
    msg: String,
}

impl ParseError {
    pub(crate) fn new<S: ToString>(kind: ParseErrorKind, msg: S) -> Self {
        Self {
            kind,
            msg: msg.to_string(),
        }
    }
}

/// The kind of `ParseError`.
#[derive(Debug)]
pub enum ParseErrorKind {
    /// Error parsing flat keys.
    FlatKey,
    /// Error parsing gron.
    Gron,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlatKey => write!(f, "flat key"),
            Self::Gron => write!(f, "gron"),
        }
    }
}
