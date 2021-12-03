use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
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
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl Error {
    pub(crate) fn new<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Message(s.as_ref().into())
    }

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
