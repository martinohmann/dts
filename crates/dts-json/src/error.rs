//! Defines the `Error` and `Result` types used by this crate.

use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error returned by all fallible operations within this crate.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// Represents a generic error message.
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Error
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::new(msg)
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::new(msg)
    }
}
