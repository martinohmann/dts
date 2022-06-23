//! Provides functionality to filter a `serde_json::Value` based on a filter expression.

use crate::Result;
use serde_json::Value;

#[cfg(feature = "jaq")]
pub mod jaq;
#[cfg(not(feature = "jaq"))]
pub mod jq;

#[cfg(feature = "jaq")]
use jaq::Filter as FilterImpl;
#[cfg(not(feature = "jaq"))]
use jq::Filter as FilterImpl;

/// A jq-like filter for transforming a `Value` into a different `Value` based on the contents of
/// a filter expression.
///
/// This can be used to transform a `Value` using a `jq` expression.
///
/// ## Example
///
/// ```
/// use dts::filter::Filter;
/// use serde_json::json;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!([5, 4, 10]);
///
/// let filter = Filter::new("map(select(. > 5))")?;
/// let result = filter.apply(value)?;
///
/// assert_eq!(result, json!([10]));
/// #   Ok(())
/// # }
/// ```
pub struct Filter {
    inner: FilterImpl,
}

impl Filter {
    /// Constructs the filter from the `&str` expression.
    ///
    /// Depending on the underlying implementation this may return an error if parsing the
    /// expression fails.
    pub fn new(expr: &str) -> Result<Filter> {
        let inner = FilterImpl::new(expr)?;
        Ok(Filter { inner })
    }

    /// Applies the filter to a `Value` and returns the result.
    pub fn apply(&self, value: Value) -> Result<Value> {
        self.inner.apply(value)
    }
}
