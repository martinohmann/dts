use crate::parsers::ParseError;
use thiserror::Error;

/// The error returned by all fallible operations within this module.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum TransformError {
    /// Represents an unknown transformation key. This usually indicates incorrect user input.
    #[error("unknown transformation: {0}")]
    UnknownTransformation(String),
    /// A jsonpath transformation is missing the filter query.
    #[error("jsonpath expects a filter query")]
    JSONPathQueryExpected,
    /// Represents a error while parsing the query of jsonpath filter transformation.
    #[error("failed to parse jsonpath query:\n{0}")]
    JSONPathParseError(String),
    /// Represents a parse error that happens during a transformation operation.
    #[error(transparent)]
    ParseError(#[from] ParseError),
}
