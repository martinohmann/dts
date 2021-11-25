//! Provides the value type that is used internally.

use crate::ValueExt;
use std::cmp::Ordering;

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

    fn primitives_first(&mut self) -> &Value {
        if let Some(array) = self.as_array_mut() {
            let mut sortable: Vec<SortableValue> = array
                .iter_mut()
                .map(ValueExt::primitives_first)
                .map(SortableValue)
                .collect();

            sortable.sort();

            *array = sortable.into_iter().map(|v| v.0.clone()).collect();
        } else if let Some(object) = self.as_object_mut() {
            let mut sortable: Vec<(&String, SortableValue)> = object
                .iter_mut()
                .map(|(k, v)| (k, v.primitives_first()))
                .map(|(k, v)| (k, SortableValue(v)))
                .collect();

            sortable.sort_by(|a, b| a.1.cmp(&b.1));

            *object = sortable
                .into_iter()
                .map(|(k, v)| (k.clone(), v.0.clone()))
                .collect()
        }

        self
    }

    fn deep_merge(&mut self, other: &mut Value) {
        match (self, other) {
            (Value::Object(lhs), Value::Object(rhs)) => {
                for (key, value) in rhs.iter_mut() {
                    lhs.entry(key)
                        .and_modify(|lhs| lhs.deep_merge(value))
                        .or_insert_with(|| std::mem::replace(value, Value::Null));
                }
            }
            (Value::Array(lhs), Value::Array(rhs)) => {
                lhs.resize(lhs.len().max(rhs.len()), Value::Null);

                for (i, value) in rhs.iter_mut().enumerate() {
                    lhs[i].deep_merge(value);
                }
            }
            (_, Value::Null) => (),
            (lhs, rhs) => *lhs = std::mem::replace(rhs, Value::Null),
        }
    }
}

#[derive(PartialEq, Eq)]
struct SortableValue<'a>(&'a Value);

impl<'a> PartialOrd for SortableValue<'a> {
    fn partial_cmp(&self, other: &SortableValue) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for SortableValue<'a> {
    fn cmp(&self, other: &SortableValue) -> Ordering {
        // Sort order: primitives, arrays, objects. Original order is preserved as much as possible
        // by avoiding to compare the values wrapped by each variant directly.
        match (self.0, other.0) {
            (Value::Array(_), Value::Array(_)) => Ordering::Equal,
            (Value::Array(_), Value::Object(_)) => Ordering::Less,
            (Value::Array(_), _) => Ordering::Greater,
            (Value::Object(_), Value::Object(_)) => Ordering::Equal,
            (Value::Object(_), Value::Array(_)) => Ordering::Greater,
            (Value::Object(_), _) => Ordering::Greater,
            (_, Value::Array(_)) => Ordering::Less,
            (_, Value::Object(_)) => Ordering::Less,
            (_, _) => Ordering::Equal,
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

    #[test]
    fn test_primitives_first() {
        assert_eq!(
            json!(["one", {"two": "three"}, [{"four": [{"five": "six"}, "seven"]}, "eight"], "nine"]).primitives_first(),
            &json!(["one", "nine", ["eight", {"four": ["seven", {"five": "six"}]}], {"two": "three"}])
        );
    }

    #[test]
    fn test_primitives_first_object() {
        // We are comparing the JSON string representation here to assert that objects have been
        // moved to the end. Comparing the maps directly will not work as they are assumed to be
        // the same with the order ignored.
        let expected_value =
            json!({"seven": "eight", "one": {"five": "six", "two": {"three": "four"}}});
        let expected = expected_value.to_string();

        let mut value = json!({"one": {"two": {"three": "four"}, "five": "six"}, "seven": "eight"});
        value.primitives_first();
        let result = value.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_primitives_first_no_change() {
        assert_eq!(
            json!({"foo": "bar"}).primitives_first(),
            &json!({"foo": "bar"})
        );
        assert_eq!(
            json!(["foo", "bar"]).primitives_first(),
            &json!(["foo", "bar"])
        );
        assert_eq!(json!("foo").primitives_first(), &json!("foo"));
    }
}
