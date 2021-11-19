//! Defines the `Error` and `Result` types used by this crate.

use crate::Encoding;
use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error returned by all fallible operations within this crate.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// A generic error message.
    #[error("{0}")]
    Message(String),

    /// Indicates an error at a specific row of input or output data.
    #[error("error at row index {0}: {1}")]
    AtRowIndex(usize, String),

    /// Indicates that deserialization is not supported for the given Encoding.
    #[error("deserializing {0} is not supported")]
    DeserializeUnsupported(Encoding),

    /// Indicates that serialization is not supported for the given Encoding.
    #[error("serializing to {0} is not supported")]
    SerializeUnsupported(Encoding),

    /// IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Error emitted by serde_yaml.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    /// Error emitted by serde_json.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Error emitted by json5.
    #[error(transparent)]
    Json5(#[from] json5::Error),

    /// Error emitted by deser_hjson.
    #[error(transparent)]
    Hjson(#[from] deser_hjson::Error),

    /// Error emitted by ron.
    #[error(transparent)]
    Ron(#[from] ron::Error),

    /// Serialization error emitted by toml.
    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    /// Deserialization error emitted by toml.
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    /// Error emitted by csv.
    #[error(transparent)]
    Csv(#[from] csv::Error),

    /// Error emitted by serde_pickle.
    #[error(transparent)]
    Pickle(#[from] serde_pickle::Error),

    /// Error emitted by serde_qs.
    #[error(transparent)]
    QueryString(#[from] serde_qs::Error),

    /// Error emitted by serde_xml.
    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    /// Error emitted by regex.
    #[error(transparent)]
    Regex(#[from] regex::Error),

    /// Error emitted by ureq.
    #[error(transparent)]
    Ureq(#[from] ureq::Error),

    /// Error emitted by glob.
    #[error(transparent)]
    GlobPattern(#[from] glob::PatternError),

    /// JsonPath query parse error.
    #[error("failed to parse jsonpath query:\n{0}")]
    JsonPath(String),
}

impl Error {
    pub(crate) fn new<T>(message: T) -> Self
    where
        T: AsRef<str>,
    {
        Self::Message(message.as_ref().to_string())
    }

    pub(crate) fn at_row_index<T>(pos: usize, message: T) -> Self
    where
        T: AsRef<str>,
    {
        Self::AtRowIndex(pos, message.as_ref().to_string())
    }
}
