//! Extension methods for `serde_json::Value`.

use serde_json::{Map, Value};
use std::fmt;
use std::iter;

/// A trait to add extension methods to `serde_json::Value`.
pub trait ValueExt {
    /// Converts value into an array. If the value is of variant `Value::Array`, the wrapped value
    /// will be returned. Otherwise the result is a `Vec` which contains the `Value`.
    fn into_array(self) -> Vec<Value>;

    /// Converts value into an object. If the value is of variant `Value::Object`, the wrapped value
    /// will be returned. Otherwise the result is a `Map` which contains a single entry with the
    /// provided key.
    fn into_object<K>(self, key: K) -> Map<String, Value>
    where
        K: fmt::Display;

    /// Converts the value to its string representation but ensures that the resulting string is
    /// not quoted.
    fn into_string(self) -> String;

    /// Deep merges `other` into `self`, replacing all values in `other` that were merged into
    /// `self` with `Value::Null`.
    fn deep_merge(&mut self, other: &mut Value);
}

impl ValueExt for Value {
    fn into_array(self) -> Vec<Value> {
        match self {
            Value::Array(array) => array,
            value => vec![value],
        }
    }

    fn into_object<K>(self, key: K) -> Map<String, Value>
    where
        K: fmt::Display,
    {
        match self {
            Value::Object(object) => object,
            value => Map::from_iter(iter::once((key.to_string(), value))),
        }
    }

    fn into_string(self) -> String {
        match self {
            Value::String(s) => s,
            value => value.to_string(),
        }
    }

    fn deep_merge(&mut self, other: &mut Value) {
        match (self, other) {
            (Value::Object(lhs), Value::Object(rhs)) => {
                rhs.iter_mut().for_each(|(key, value)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge(value))
                        .or_insert_with(|| value.take());
                });
            }
            (Value::Array(lhs), Value::Array(rhs)) => {
                lhs.resize(lhs.len().max(rhs.len()), Value::Null);

                rhs.iter_mut()
                    .enumerate()
                    .for_each(|(i, rhs)| lhs[i].deep_merge(rhs));
            }
            (_, Value::Null) => (),
            (lhs, rhs) => *lhs = rhs.take(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_into_array() {
        assert_eq!(json!("foo").into_array(), vec![json!("foo")]);
        assert_eq!(json!(["foo"]).into_array(), vec![json!("foo")]);
        assert_eq!(
            json!({"foo": "bar"}).into_array(),
            vec![json!({"foo": "bar"})]
        );
    }

    #[test]
    fn test_into_object() {
        assert_eq!(
            json!("foo").into_object("the-key"),
            Map::from_iter(iter::once(("the-key".into(), json!("foo"))))
        );
        assert_eq!(
            json!(["foo", "bar"]).into_object("the-key"),
            Map::from_iter(iter::once(("the-key".into(), json!(["foo", "bar"]))))
        );
        assert_eq!(
            json!({"foo": "bar"}).into_object("the-key"),
            Map::from_iter(iter::once(("foo".into(), json!("bar"))))
        );
    }

    #[test]
    fn test_into_string() {
        assert_eq!(
            json!({"foo": "bar"}).into_string(),
            String::from(r#"{"foo":"bar"}"#)
        );
        assert_eq!(
            json!(["foo", "bar"]).into_string(),
            String::from(r#"["foo","bar"]"#)
        );
        assert_eq!(json!("foo").into_string(), String::from("foo"));
        assert_eq!(json!(true).into_string(), String::from("true"));
        assert_eq!(json!(1).into_string(), String::from("1"));
        assert_eq!(Value::Null.into_string(), String::from("null"));
    }
}
