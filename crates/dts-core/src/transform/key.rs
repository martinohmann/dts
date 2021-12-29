use crate::parsers::flat_key::StringKeyParts;
use dts_json::{Map, Value};
use std::collections::BTreeMap;

pub struct KeyFlattener<'a> {
    prefix: &'a str,
    stack: StringKeyParts,
}

impl<'a> KeyFlattener<'a> {
    pub fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            stack: StringKeyParts::new(),
        }
    }

    pub fn flatten(&mut self, value: Value) -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        self.stack.push_ident(self.prefix);
        self.flatten_value(&mut map, value);
        self.stack.pop();
        map
    }

    fn flatten_value(&mut self, map: &mut BTreeMap<String, Value>, value: Value) {
        match value {
            Value::Array(array) => {
                map.insert(self.key(), Value::Array(Vec::new()));
                for (index, value) in array.into_iter().enumerate() {
                    self.stack.push_index(index);
                    self.flatten_value(map, value);
                    self.stack.pop();
                }
            }
            Value::Object(object) => {
                map.insert(self.key(), Value::Object(Map::new()));
                for (key, value) in object.into_iter() {
                    self.stack.push_ident(&key);
                    self.flatten_value(map, value);
                    self.stack.pop();
                }
            }
            value => {
                map.insert(self.key(), value);
            }
        }
    }

    fn key(&self) -> String {
        self.stack.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_key_flattener() {
        let value = json!({"foo": {"bar": ["baz", "qux"]}});

        let mut flattener = KeyFlattener::new("data");
        let value = Value::Object(Map::from_iter(flattener.flatten(value)));

        assert_eq!(
            value,
            json!({
                "data": {},
                "data.foo": {},
                "data.foo.bar": [],
                "data.foo.bar[0]": "baz",
                "data.foo.bar[1]": "qux"
            })
        );
    }
}
