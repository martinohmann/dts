#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
mod path;

pub use error::{Error, Result};
pub use parser::parse;

use dts_json::Value;
use path::{JsonPath, Selector as SelectorTrait, Values};

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
        let values = Values::new_root(value);
        self.path.select(&values)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[test]
    fn test_selector() {
        let selector = Selector::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        assert_eq!(selector.select(&value), vec![&json!(2), &json!(3)]);
    }
}
