use std::fmt::{self, Display};
use thiserror::Error;

/// Error emitted by all parsers in this module.
#[derive(Error, Debug)]
#[error("failed to parse {kind}:\n{msg}")]
pub struct ParseError {
    kind: ParseErrorKind,
    msg: String,
}

impl ParseError {
    pub(crate) fn new<T>(kind: ParseErrorKind, msg: T) -> ParseError
    where
        T: Display,
    {
        ParseError {
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

impl Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::FlatKey => write!(f, "flat key"),
            ParseErrorKind::Gron => write!(f, "gron"),
        }
    }
}
