use crate::parsers::flat_key::StringKeyParts;
use crate::Value;
use serde_json::Map;
use std::collections::BTreeMap;

pub struct KeyFlattener<'a> {
    value: &'a Value,
    prefix: &'a str,
    stack: StringKeyParts,
}

impl<'a> KeyFlattener<'a> {
    pub fn new(value: &'a Value, prefix: &'a str) -> Self {
        Self {
            value,
            prefix,
            stack: StringKeyParts::new(),
        }
    }

    pub fn flatten(&mut self) -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        self.stack.push_ident(self.prefix);
        self.map_value(&mut map, self.value);
        self.stack.pop();
        map
    }

    fn map_value(&mut self, map: &mut BTreeMap<String, Value>, value: &'a Value) {
        match value {
            Value::Array(array) => {
                map.insert(self.key(), Value::Array(Vec::new()));
                for (index, value) in array.iter().enumerate() {
                    self.stack.push_index(index);
                    self.map_value(map, value);
                    self.stack.pop();
                }
            }
            Value::Object(object) => {
                map.insert(self.key(), Value::Object(Map::new()));
                for (key, value) in object.iter() {
                    self.stack.push_ident(key);
                    self.map_value(map, value);
                    self.stack.pop();
                }
            }
            value => {
                map.insert(self.key(), value.clone());
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
        let value = Value::Object(Map::from_iter(flattener.flatten()));

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
