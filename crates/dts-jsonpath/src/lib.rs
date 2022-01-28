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

#[derive(Debug, Clone)]
pub struct JsonPath {
    selectors: Vec<Selector>,
}

impl JsonPath {
    pub fn new(query: &str) -> Result<JsonPath> {
        let selectors = parse(query)?;
        Ok(JsonPath { selectors })
    }

    pub fn find<'a>(&'a self, value: &'a Value) -> Vec<&'a Value> {
        compile(&self.selectors, value).select(value)
    }

    pub fn select(&self, value: Value) -> Value {
        self.find(&value).clone().into()
    }

    pub fn visit<F>(&self, value: &mut Value, f: F)
    where
        F: FnMut(&mut Value),
    {
        let root = value.clone();
        compile(&self.selectors, &root).visit(value, f);
    }

    pub fn mutate<F>(&self, mut value: Value, f: F) -> Value
    where
        F: Fn(Value) -> Value,
    {
        self.visit(&mut value, |value| *value = f(value.clone()));
        value
    }

    pub fn replace(&self, value: Value, replacement: Value) -> Value {
        self.mutate(value, |_| replacement.clone())
    }

    pub fn replace_with<F>(&self, value: Value, f: F) -> Value
    where
        F: Fn() -> Value,
    {
        self.mutate(value, |_| f())
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
