//! Provides a `JsonPathSelector` type.

// This module is here because the types in the `jsonpath_rust` crate are neither `Clone` nor
// `Debug` which makes it hard to integrate them into the `Transformation` type.
//
// The `JsonPathSelector` type is a wrapper around `jsonpath_rust::JsonPathInst` and
// `jsonpath_rust::JsonPathFinder` to make working with it easier at the cost of an unnecessary
// clone and parse operation.
//
// This cannot be avoided as we want to separate the time when the query is validated and parsed
// from the time where it is used to have a consistent UX.
use crate::{Error, Result};
use dts_json::Value;
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use std::fmt;
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
pub struct JsonPathSelector {
    query: String,
    finder: JsonPathFinder,
}

impl JsonPathSelector {
    /// Creates a new `JsonPathSelector` for the provided `query`.
    ///
    /// ## Errors
    ///
    /// Returns an error if query is malformed and cannot be parsed.
    pub fn new(query: &str) -> Result<Self> {
        let inst = JsonPathInst::from_str(query)
            .map_err(|err| Error::new(format!("Failed to parse jsonpath query:\n{}", err)))?;

        let finder = JsonPathFinder::new(serde_json::Value::Null.into(), inst.into());

        Ok(JsonPathSelector {
            query: query.to_owned(),
            finder,
        })
    }

    /// Applies the `JsonPathSelector` to the `Value` and returns the selected elements as a new
    /// `Value`.
    pub fn select(&self, value: Value) -> Value {
        self.clone().select_mut(value)
    }

    fn select_mut(&mut self, value: Value) -> Value {
        self.finder.set_json(Box::new(value.into()));
        self.finder.find().into()
    }
}

impl Clone for JsonPathSelector {
    fn clone(&self) -> Self {
        JsonPathSelector::new(&self.query).unwrap()
    }
}

impl fmt::Debug for JsonPathSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonPathSelector")
            .field("query", &self.query)
            .finish()
    }
}

impl FromStr for JsonPathSelector {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPathSelector::new(s)
    }
}
