//! Provides a `JsonPathSelector` type.

use crate::{Error, Result};
use dts_json::Value;
use jsonpath_lib::{Compiled as CompiledJsonPath, SelectorMut};
use serde_json::Value as JsonValue;
use std::cell::RefCell;
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

/// A json selector that can mutate parts of a value in place.
pub struct JsonPathMutator {
    selector: RefCell<SelectorMut>,
}

impl JsonPathMutator {
    /// Creates a new `JsonPathMutator`.
    pub fn new(query: &str) -> Result<Self> {
        let mut selector = SelectorMut::new();

        selector
            .str_path(&query)
            .map_err(|err| Error::new(format!("Failed to parse jsonpath query:\n\n{}", err)))?;

        Ok(JsonPathMutator {
            selector: RefCell::new(selector),
        })
    }

    /// Mutate the parts of the value that are matched by the query and return the mutated result.
    pub fn mutate<F>(&self, value: Value, mut replacer: F) -> Value
    where
        F: FnMut(Value) -> Value,
    {
        self.selector
            .borrow_mut()
            .value(value.into())
            .replace_with(&mut |value: JsonValue| Some((replacer)(value.into()).into()))
            .unwrap()
            .take()
            .map(Into::into)
            .unwrap_or_default()
    }
}
