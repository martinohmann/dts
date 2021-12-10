use crate::parsers::ParseError;
use std::fmt::Display;
use thiserror::Error;

/// The error returned by all fallible operations within this module.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum TransformError {
    /// Represents an unknown transformation key. This usually indicates incorrect user input.
    #[error("Unknown transformation `{0}`")]
    UnknownTransformation(String),

    /// A transformation requires a value.
    #[error("Transformation `{0}` requires a value")]
    ValueRequired(String),

    /// Represents a error while parsing the query of jsonpath filter transformation.
    #[error("Failed to parse JSONPath query:\n{0}")]
    JSONPathParseError(String),

    /// Represents an invalid sort order.
    #[error("Invalid sort order `{0}`")]
    InvalidSortOrder(String),

    /// Represents a parse error that happens during a transformation operation.
    #[error("Parse error during data transformation")]
    ParseError(#[from] ParseError),

    /// Represents an error while compiling a regex.
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    /// Represents an error while parsing an integer.
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl TransformError {
    pub(crate) fn invalid_sort_order<T>(order: T) -> Self
    where
        T: Display,
    {
        TransformError::InvalidSortOrder(order.to_string())
    }

    pub(crate) fn unknown_transformation<T>(trans: T) -> Self
    where
        T: Display,
    {
        TransformError::UnknownTransformation(trans.to_string())
    }

    pub(crate) fn value_required<T>(trans: T) -> Self
    where
        T: Display,
    {
        TransformError::ValueRequired(trans.to_string())
    }
}
