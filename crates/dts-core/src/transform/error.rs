use std::fmt::Display;
use thiserror::Error;

/// The error returned by all fallible operations within this module.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum TransformError {
    /// Represents a error while parsing the query of jsonpath filter transformation.
    #[error("Failed to parse JSONPath query:\n{0}")]
    JSONPathParseError(String),

    /// Represents an invalid sort order.
    #[error("Invalid sort order `{0}`")]
    InvalidSortOrder(String),

    /// Represents an error while compiling a regex.
    #[error(transparent)]
    RegexError(#[from] regex::Error),
}

impl TransformError {
    pub(crate) fn invalid_sort_order<T>(order: T) -> Self
    where
        T: Display,
    {
        TransformError::InvalidSortOrder(order.to_string())
    }
}
