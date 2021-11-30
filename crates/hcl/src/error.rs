use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("HCL parse error:\n{0}")]
    ParseError(String),
    #[error("{0}")]
    Message(String),
    #[error("EOF")]
    Eof,
    #[error("Syntax error")]
    Syntax,
    #[error("Token expected `{0}`")]
    TokenExpected(String),
}

impl Error {
    pub(crate) fn token_expected<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::TokenExpected(s.as_ref().into())
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
