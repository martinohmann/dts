//! Provides a `JsonPathSelector` type.

use crate::{Error, Result};
use dts_json::Value;
use jsonpath_lib::Compiled as CompiledJsonPath;
use std::str::FromStr;

/// A jsonpath selector.
///
/// ## Example
///
/// ```
/// use dts_core::transform::jsonpath::JsonPathSelector;
/// use dts_json::json;
/// use std::str::FromStr;
/// # use pretty_assertions::assert_eq;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!({
///   "orders": [
///     {"id": 1, "active": true},
///     {"id": 2},
///     {"id": 3},
///     {"id": 4, "active": true}
///   ]
/// });
///
/// let selector = JsonPathSelector::new("$.orders[?(@.active)].id")?;
///
/// assert_eq!(selector.select(value), json!([1, 4]));
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct JsonPathSelector {
    compiled: CompiledJsonPath,
}

impl JsonPathSelector {
    /// Creates a new `JsonPathSelector` for the provided `query`.
    ///
    /// ## Errors
    ///
    /// Returns an error if query is malformed and cannot be parsed.
    pub fn new(query: &str) -> Result<Self> {
        let compiled = CompiledJsonPath::compile(query)
            .map_err(|err| Error::new(format!("Failed to parse jsonpath query:\n\n{}", err)))?;

        Ok(JsonPathSelector { compiled })
    }

    /// Applies the `JsonPathSelector` to the `Value` and returns the selected elements as a new
    /// `Value`.
    pub fn select(&self, value: Value) -> Value {
        self.compiled.select(&value.into()).unwrap().into()
    }
}

impl FromStr for JsonPathSelector {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPathSelector::new(s)
    }
}
