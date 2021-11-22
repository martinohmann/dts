use crate::parsers::flat_key::KeyParts;
use crate::Value;
use serde_json::Map;
use std::collections::BTreeMap;

pub struct KeyFlattener<'a> {
    value: &'a Value,
    prefix: &'a str,
    map: BTreeMap<String, Value>,
    stack: KeyParts,
}

impl<'a> KeyFlattener<'a> {
    pub fn new(value: &'a Value, prefix: &'a str) -> Self {
        Self {
            value,
            prefix,
            map: BTreeMap::new(),
            stack: KeyParts::new(),
        }
    }

    pub fn flatten(&mut self) -> BTreeMap<String, Value> {
        self.stack.push_ident(self.prefix);
        self.map_value(self.value);
        self.stack.pop();
        self.map.clone()
    }

    fn map_value(&mut self, value: &'a Value) {
        match value {
            Value::Array(array) => {
                self.map.insert(self.key(), Value::Array(Vec::new()));
                for (index, value) in array.iter().enumerate() {
                    self.stack.push_index(index);
                    self.map_value(value);
                    self.stack.pop();
                }
            }
            Value::Object(object) => {
                self.map.insert(self.key(), Value::Object(Map::new()));
                for (key, value) in object.iter() {
                    self.stack.push_ident(key);
                    self.map_value(value);
                    self.stack.pop();
                }
            }
            value => {
                self.map.insert(self.key(), value.clone());
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_key_flattener() {
        let value = json!({"foo": {"bar": ["baz", "qux"]}});

        let mut flattener = KeyFlattener::new(&value, "data");
        let value = Value::Object(Map::from_iter(flattener.flatten().into_iter()));

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
