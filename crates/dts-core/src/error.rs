//! Defines the `Error` and `Result` types used by this crate.

use crate::{parsers::ParseError, transform::TransformError, Encoding};
use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error returned by all fallible operations within this crate.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// Represents a generic error message.
    #[error("{0}")]
    GenericError(String),

    /// Represents errors of operations that are not supported by a certain encoding.
    #[error("Operation is not supported for encoding `{0}`")]
    UnsupportedEncoding(Encoding),

    /// Error emitted by parsers from this crate.
    #[error(transparent)]
    ParseError(#[from] ParseError),

    /// Error emitted by the transform module of this crate.
    #[error(transparent)]
    TransformError(#[from] TransformError),

    /// Represents generic IO errors.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Represents an invalid glob pattern.
    #[error("Invalid glob pattern `{pattern}`")]
    GlobPatternError {
        /// The pattern that caused the error.
        pattern: String,
        /// The underlying error.
        source: glob::PatternError,
    },

    /// Represents an error fetching a remote data source.
    #[error("Unable to fetch remote data source")]
    RequestError(#[from] ureq::Error),

    /// Error emitted by serde_yaml.
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),

    /// Error emitted by serde_json.
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// Error emitted by json5.
    #[error(transparent)]
    Json5Error(#[from] json5::Error),

    /// Serialization error emitted by toml.
    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),

    /// Deserialization error emitted by toml.
    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),

    /// Error emitted by csv.
    #[error(transparent)]
    CsvError(#[from] csv::Error),

    /// Indicates an error at a specific row of input or output data.
    #[error("Error at row index {0}: {1}")]
    CsvRowError(usize, String),

    /// Error emitted by serde_qs.
    #[error(transparent)]
    QueryStringError(#[from] serde_qs::Error),

    /// Error emitted by serde_xml.
    #[error(transparent)]
    XmlError(#[from] serde_xml_rs::Error),

    /// Error emitted by hcl.
    #[error(transparent)]
    HclError(#[from] hcl::Error),
}

impl Error {
    pub(crate) fn new<S: ToString>(message: S) -> Self {
        Self::GenericError(message.to_string())
    }
}
