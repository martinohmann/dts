//! Object key transformation utilities.

use crate::{
    parsers::flat_key::{self, KeyPart, KeyParts, StringKeyParts},
    value::ValueExt,
};
use rayon::prelude::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::iter;

/// Flattens value to an object with flat keys.
///
/// ## Examples
///
/// Nested map with array:
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::key::flatten_keys;
/// use serde_json::json;
///
/// let value = json!({"foo": {"bar": ["baz", "qux"]}});
///
/// let value = flatten_keys(value, "data");
///
/// assert_eq!(
///     value,
///     json!({
///         "data": {},
///         "data.foo": {},
///         "data.foo.bar": [],
///         "data.foo.bar[0]": "baz",
///         "data.foo.bar[1]": "qux"
///     })
/// );
/// ```
///
/// Array value with prefix "array":
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::key::flatten_keys;
/// use serde_json::json;
///
/// let value = json!(["foo", "bar", "baz"]);
///
/// let value = flatten_keys(value, "array");
///
/// assert_eq!(
///     value,
///     json!({
///         "array": [],
///         "array[0]": "foo",
///         "array[1]": "bar",
///         "array[2]": "baz"
///     })
/// );
/// ```
///
/// Single primitive value:
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::key::flatten_keys;
/// use serde_json::json;
///
/// let value = json!("foo");
///
/// assert_eq!(flatten_keys(value, "data"), json!({"data": "foo"}));
/// ```
pub fn flatten_keys<P>(value: Value, prefix: P) -> Value
where
    P: AsRef<str>,
{
    let mut flattener = KeyFlattener::new(prefix.as_ref());
    Value::Object(Map::from_iter(flattener.flatten(value)))
}

/// Recursively expands flat keys to nested objects.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::key::expand_keys;
/// use serde_json::json;
///
/// let value = json!([{"foo.bar": 1, "foo[\"bar-baz\"]": 2}]);
/// let expected = json!([{"foo": {"bar": 1, "bar-baz": 2}}]);
///
/// assert_eq!(expand_keys(value), expected);
/// ```
pub fn expand_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => object
            .into_iter()
            .collect::<Vec<(String, Value)>>()
            .into_par_iter()
            .map(|(key, value)| match flat_key::parse(&key).ok() {
                Some(mut parts) => {
                    parts.reverse();
                    expand_key_parts(&mut parts, value)
                }
                None => Value::Object(Map::from_iter(iter::once((key, value)))),
            })
            .reduce(
                || Value::Null,
                |mut a, mut b| {
                    a.deep_merge(&mut b);
                    a
                },
            ),
        Value::Array(array) => Value::Array(array.into_iter().map(expand_keys).collect()),
        value => value,
    }
}

fn expand_key_parts(parts: &mut KeyParts, value: Value) -> Value {
    match parts.pop() {
        Some(key) => match key {
            KeyPart::Ident(ident) => {
                let mut object = Map::with_capacity(1);
                object.insert(ident, expand_key_parts(parts, value));
                Value::Object(object)
            }
            KeyPart::Index(index) => {
                let mut array = vec![Value::Null; index + 1];
                array[index] = expand_key_parts(parts, value);
                Value::Array(array)
            }
        },
        None => value,
    }
}

struct KeyFlattener<'a> {
    prefix: &'a str,
    stack: StringKeyParts,
}

impl<'a> KeyFlattener<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            stack: StringKeyParts::new(),
        }
    }

    fn flatten(&mut self, value: Value) -> BTreeMap<String, Value> {
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
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_expand_keys() {
        let value = json!({
            "data": {},
            "data.foo": {},
            "data.foo.bar": [],
            "data.foo.bar[0]": "baz",
            "data.foo.bar[1]": "qux"
        });

        assert_eq!(
            expand_keys(value),
            json!({"data": {"foo": {"bar": ["baz", "qux"]}}})
        );
    }

    #[test]
    fn test_flatten_keys() {
        let value = json!({"foo": {"bar": ["baz", "qux"]}});

        assert_eq!(
            flatten_keys(value, "data"),
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
