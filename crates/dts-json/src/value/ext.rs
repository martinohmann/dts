use super::Value;

impl Value {
    /// Converts value into an array. If the value is of variant `Value::Array`, the wrapped value
    /// will be returned. Otherwise the result is a `Vec` which contains the `Value`.
    pub fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(array) => array.clone(),
            value => vec![value.clone()],
        }
    }

    /// Converts the value to its string representation but ensures that the resulting string is
    /// not quoted.
    pub fn to_string_unquoted(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            value => value.to_string(),
        }
    }

    /// Deep merges `other` into `self`, replacing all values in `other` that were merged into
    /// `self` with `Value::Null`.
    pub fn deep_merge(&mut self, other: &mut Value) {
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

    /// Returns true if `self` is `Value::Null` or an empty array or map.
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Null => true,
            Value::Array(array) if array.is_empty() => true,
            Value::Object(object) if object.is_empty() => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_array() {
        assert_eq!(json!("foo").to_array(), vec![json!("foo")]);
        assert_eq!(json!(["foo"]).to_array(), vec![json!("foo")]);
        assert_eq!(
            json!({"foo": "bar"}).to_array(),
            vec![json!({"foo": "bar"})]
        );
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
