//! Provides functionality to filter a `serde_json::Value` based on a filter expression.

use crate::Result;
use serde_json::Value;

#[cfg(feature = "jaq")]
pub mod jaq;
#[cfg(not(feature = "jaq"))]
pub mod jq;

#[cfg(feature = "jaq")]
pub use jaq::Filter as FilterImpl;
#[cfg(not(feature = "jaq"))]
pub use jq::Filter as FilterImpl;

/// A filter for transforming a `Value` into a different `Value` based on the contents of a filter
/// expression.
pub trait Filter {
    /// The type created by the `parse` method.
    type Item: Filter;

    /// Parses a `&str` expression and constructs the filter.
    fn parse(expr: &str) -> Result<Self::Item>;

    /// Applies the filter to a `Value` and returns the result.
    fn apply(&self, value: Value) -> Result<Value>;
}
