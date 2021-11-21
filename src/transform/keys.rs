use crate::Value;
use serde_json::Map;
use std::collections::BTreeMap;

enum Key<'a> {
    Index(usize),
    Ident(&'a str),
}

impl<'a> ToString for Key<'a> {
    fn to_string(&self) -> String {
        match self {
            Key::Index(index) => format!("[{}]", index),
            Key::Ident(key) => {
                let no_escape = key
                    .chars()
                    .all(|c| c == '_' || c.is_numeric() || c.is_alphabetic());

                if no_escape {
                    key.to_string()
                } else {
                    format!("[\"{}\"]", key.escape_default().collect::<String>())
                }
            }
        }
    }
}

pub struct KeyFlattener<'a> {
    value: &'a Value,
    prefix: &'a str,
    map: BTreeMap<String, Value>,
    stack: Vec<String>,
}

impl<'a> KeyFlattener<'a> {
    pub fn new(value: &'a Value, prefix: &'a str) -> Self {
        Self {
            value,
            prefix,
            map: BTreeMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn flatten(&mut self) -> BTreeMap<String, Value> {
        self.map_value(self.value);
        self.map.clone()
    }

    fn map_value(&mut self, value: &Value) {
        match value {
            Value::Array(array) => {
                self.map.insert(self.key(), Value::Array(Vec::new()));
                for (index, value) in array.iter().enumerate() {
                    self.stack.push(Key::Index(index).to_string());
                    self.map_value(value);
                    self.stack.pop();
                }
            }
            Value::Object(object) => {
                self.map.insert(self.key(), Value::Object(Map::new()));
                for (key, value) in object.iter() {
                    self.stack.push(Key::Ident(key).to_string());
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
        let acc = Key::Ident(self.prefix).to_string();
        self.stack.iter().fold(acc, |mut acc, key| {
            if !acc.is_empty() && !key.starts_with('[') {
                acc.push('.');
            }
            acc.push_str(key);
            acc
        })
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
