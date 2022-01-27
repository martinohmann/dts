#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
pub mod path;

pub use error::{Error, Result};

use dts_json::Value;
use parser::ast;
use path::{Select, Visitor};

pub struct Selector {
    ast: Vec<ast::Selector>,
}

impl Selector {
    pub fn new(query: &str) -> Result<Selector> {
        let ast = parser::parse(query)?;
        Ok(Selector { ast })
    }

    pub fn find<'a>(&'a self, value: &'a Value) -> Vec<&'a Value> {
        path::compile(&self.ast, value).select(value)
    }

    pub fn select(&self, value: Value) -> Value {
        self.find(&value).iter().cloned().collect()
    }

    pub fn mutate<F>(&self, mut value: Value, f: &mut F) -> Value
    where
        F: FnMut(&mut Value),
    {
        let root = value.clone();
        let path = path::compile(&self.ast, &root);
        let mut visitor = Visitor::new(vec![&path], f);
        visitor.visit(&mut value);
        value
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[test]
    fn test_select() {
        let selector = Selector::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        assert_eq!(selector.select(value), json!([2, 3]));
    }

    #[test]
    fn test_mutate() {
        let selector = Selector::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        let result = selector.mutate(value, &mut |value| {
            *value = Value::String("replaced".into());
        });
        assert_eq!(
            result,
            json!({"bar": {"baz": 1}, "foo": {"bar": "replaced", "qux": "replaced"}})
        );
    }
}
