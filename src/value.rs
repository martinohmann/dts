//! Provides the value type that is used internally.

use crate::ValueExt;

/// The type this crate uses internally to represent arbitrary data.
///
/// This is just a type alias for `serde_json::Value` as it has most of the features necessary for
/// internal data transformation.
pub type Value = serde_json::Value;

impl ValueExt for Value {
    fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(array) => array.clone(),
            value => vec![value.clone()],
        }
    }

    fn to_string_unquoted(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            value => value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_to_array() {
        assert_eq!(json!("foo").to_array(), json!(["foo"]));
        assert_eq!(json!(["foo"]).to_array(), json!(["foo"]));
        assert_eq!(json!({"foo": "bar"}).to_array(), json!([{"foo": "bar"}]));
    }

    #[test]
    fn test_to_string_unquoted() {
        assert_eq!(
            json!({"foo": "bar"}).to_string_unquoted(),
            String::from(r#"{"foo":"bar"}"#)
        );
        assert_eq!(
            json!(["foo", "bar"]).to_string_unquoted(),
            String::from(r#"["foo","bar"]"#)
        );
        assert_eq!(json!("foo").to_string_unquoted(), String::from("foo"));
        assert_eq!(json!(true).to_string_unquoted(), String::from("true"));
        assert_eq!(json!(1).to_string_unquoted(), String::from("1"));
        assert_eq!(Value::Null.to_string_unquoted(), String::from("null"));
    }
}
