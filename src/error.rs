//! Defines the `Error` and `Result` types used by this crate.

use crate::{Encoding, parsers::ParseError};
use std::error::Error as StdError;
use std::fmt::Display;
use std::io;
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

    /// Represents errors of operations that are not supported by a certain encoding.
    #[error("operation is not supported for encoding `{0}`")]
    UnsupportedEncoding(Encoding),

    /// Error emitted by parsers from this crate.
    #[error(transparent)]
    ParseError(#[from] ParseError),

    /// Represents generic IO errors.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Represents an invalid glob pattern.
    #[error("invalid glob pattern `{pattern}`")]
    GlobPatternError {
        /// The pattern that caused the error.
        pattern: String,
        /// The underlying error.
        source: glob::PatternError,
    },

    /// Represents an error fetching a remote data source.
    #[error("unable to fetch remote data source")]
    RequestError(Box<ureq::Error>),

    /// Represents errors emitted by serializers and deserializers.
    #[error(transparent)]
    Serde(Box<dyn StdError + Send + Sync>),
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Error
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }

    pub(crate) fn serde<E>(err: E) -> Error
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        Error::Serde(err.into())
    }

    pub(crate) fn io<E>(err: E) -> Error
    where
        E: Into<io::Error>,
    {
        Error::Io(err.into())
    }

    pub(crate) fn glob_pattern<T>(pattern: T, source: glob::PatternError) -> Error
    where
        T: Display,
    {
        Error::GlobPatternError {
            pattern: pattern.to_string(),
            source,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        if err.is_io() {
            Error::io(err)
        } else {
            Error::serde(err)
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        if let Some(source) = err.source() {
            if let Some(io_err) = source.downcast_ref::<io::Error>() {
                return Error::io(io_err.kind());
            }
        }

        Error::serde(err)
    }
}

impl From<json5::Error> for Error {
    fn from(err: json5::Error) -> Self {
        Error::serde(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::serde(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::serde(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        if err.is_io_error() {
            match err.into_kind() {
                csv::ErrorKind::Io(io_err) => Error::io(io_err),
                _ => unreachable!(),
            }
        } else {
            Error::serde(err)
        }
    }
}

impl From<serde_qs::Error> for Error {
    fn from(err: serde_qs::Error) -> Self {
        match err {
            serde_qs::Error::Io(io_err) => Error::io(io_err),
            other => Error::serde(other),
        }
    }
}

impl From<serde_xml_rs::Error> for Error {
    fn from(err: serde_xml_rs::Error) -> Self {
        Error::serde(err)
    }
}

impl From<hcl::Error> for Error {
    fn from(err: hcl::Error) -> Self {
        match err {
            hcl::Error::Io(io_err) => Error::io(io_err),
            other => Error::serde(other),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        Error::RequestError(Box::new(err))
    }
}
