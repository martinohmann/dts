#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
pub mod path;

pub use error::{Error, Result};

use dts_json::Value;
use parser::ast;

pub struct JsonPath {
    selectors: Vec<ast::Selector>,
}

impl JsonPath {
    pub fn new(query: &str) -> Result<JsonPath> {
        let selectors = parser::parse(query)?;
        Ok(JsonPath { selectors })
    }

    pub fn find<'a>(&'a self, value: &'a Value) -> Vec<&'a Value> {
        path::compile(&self.selectors, value).select(value)
    }

    pub fn select(&self, value: Value) -> Value {
        self.find(&value).iter().cloned().collect()
    }

    pub fn mutate<F>(&self, mut value: Value, f: F) -> Value
    where
        F: FnMut(&mut Value),
    {
        let root = value.clone();
        let path = path::compile(&self.selectors, &root);
        path.visit(&mut value, f);
        value
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
    fn test_mutate() {
        let path = JsonPath::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        let result = path.mutate(value, |value| {
            *value = Value::String("replaced".into());
        });
        assert_eq!(
            result,
            json!({"bar": {"baz": 1}, "foo": {"bar": "replaced", "qux": "replaced"}})
        );
    }
}
