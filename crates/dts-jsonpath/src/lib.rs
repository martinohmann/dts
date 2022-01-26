#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
mod path;

pub use error::{Error, Result};
pub use parser::parse;

use dts_json::Value;
use path::{JsonPath, PathPointer, PathPointerMut, PathSelector, Visitor};

pub struct Selector {
    path: JsonPath,
}

impl Selector {
    pub fn new(query: &str) -> Result<Selector> {
        let ast = parser::parse(query)?;
        let path = path::compile(ast);
        Ok(Selector { path })
    }

    pub fn select<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        let pointer = PathPointer::new(value, value);
        self.path.select(&pointer)
    }

    pub fn mutate<F>(&self, mut value: Value, f: &mut F) -> Value
    where
        F: FnMut(&mut Value),
    {
        let chain = vec![self.path.clone()];
        let mut current = value.clone();
        let mut pointer = PathPointerMut::new(&mut current, &mut value);
        let mut visitor = Visitor::new(&chain, f);
        visitor.visit(&mut pointer);
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
        assert_eq!(selector.select(&value), vec![&json!(2), &json!(3)]);
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
