#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
pub mod path;

pub use crate::error::{Error, Result};
use crate::parser::ast::Selector;
pub use crate::parser::parse;
pub use crate::path::compile;
use dts_json::Value;
use std::str::FromStr;

/// Represents a jsonpath query that can be used for filtering and mutating json values.
#[derive(Debug, Clone)]
pub struct JsonPath {
    selectors: Vec<Selector>,
}

impl JsonPath {
    /// Creates a new `JsonPath` from a query. The returned value can be used multiple times.
    ///
    /// ## Errors
    ///
    /// Returns an error if the input is not a valid jsonpath query.
    pub fn new(query: &str) -> Result<JsonPath> {
        let selectors = parse(query)?;
        Ok(JsonPath { selectors })
    }

    /// Finds all matching `Value`s in `root` and returns references to them.
    pub fn find<'a>(&'a self, root: &'a Value) -> Vec<&'a Value> {
        compile(&self.selectors, root).select(root)
    }

    /// Selects all matching `Value`s from `root` and returns copies of them.
    pub fn select(&self, root: Value) -> Value {
        self.find(&root).clone().into()
    }

    /// Recursively visits `root` and calls `f` for every matching `Value`.
    pub fn visit<F>(&self, root: &mut Value, f: F)
    where
        F: FnMut(&mut Value),
    {
        let root_ref = root.clone();
        compile(&self.selectors, &root_ref).visit(root, f);
    }

    /// Recursively visits `root` and calls `f` for every matching `Value`, producing a new
    /// `Value`.
    pub fn mutate<F>(&self, mut root: Value, f: F) -> Value
    where
        F: Fn(Value) -> Value,
    {
        self.visit(&mut root, |value| *value = f(value.clone()));
        root
    }

    /// Recursively visits `root` and replaces all matches with the `Value` returned by `f`.
    pub fn replace_with<F>(&self, value: Value, f: F) -> Value
    where
        F: Fn() -> Value,
    {
        self.mutate(value, |_| f())
    }

    /// Recursively visits `root` and replaces all matches with the `replacement`.
    pub fn replace(&self, value: Value, replacement: Value) -> Value {
        self.replace_with(value, || replacement.clone())
    }
}

impl FromStr for JsonPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPath::new(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[test]
    fn test_select() {
        let path = JsonPath::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        assert_eq!(path.select(value), json!([2, 3]));
    }

    #[test]
    fn test_replace_with() {
        let path = JsonPath::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        let result = path.replace_with(value, || Value::String("replaced".into()));
        assert_eq!(
            result,
            json!({"bar": {"baz": 1}, "foo": {"bar": "replaced", "qux": "replaced"}})
        );
    }
}
