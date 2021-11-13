//! Data transformation utilities.

use crate::{value_to_string, Error, Result, Value};
use jsonpath_rust::JsonPathQuery;
use serde_json::Map;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Filter value according to the jsonpath query.
///
/// ## Example
///
/// ```
/// use dts::transform::filter_jsonpath;
/// use serde_json::json;
/// # use pretty_assertions::assert_eq;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let mut value = json!({
///   "orders": [
///     {"id": 1, "active": true},
///     {"id": 2},
///     {"id": 3},
///     {"id": 4, "active": true}
///   ]
/// });
///
/// assert_eq!(filter_jsonpath(&mut value, "$.orders[?(@.active)].id")?, json!([1, 4]));
/// #     Ok(())
/// # }
/// ```
///
/// ## Errors
///
/// This function can fail if parsing the query fails.
///
/// ```
/// use dts::transform::filter_jsonpath;
/// use serde_json::json;
///
/// let value = json!([]);
/// assert!(filter_jsonpath(&value, "$[").is_err());
/// ```
pub fn filter_jsonpath<Q>(value: &Value, query: Q) -> Result<Value>
where
    Q: AsRef<str>,
{
    value.clone().path(query.as_ref()).map_err(Error::new)
}

/// Removes nulls, empty arrays and empty objects from value. Top level empty values are not
/// removed.
///
/// ## Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::remove_empty_values;
/// use dts::Value;
/// use serde_json::json;
///
/// let value = Value::Null;
///
/// assert_eq!(remove_empty_values(&value), Value::Null);
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::remove_empty_values;
/// use dts::Value;
/// use serde_json::json;
///
/// let mut value = json!({});
///
/// assert_eq!(remove_empty_values(&value), json!({}));
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::remove_empty_values;
/// use serde_json::json;
///
/// let value = json!(["foo", null, "bar"]);
///
/// assert_eq!(remove_empty_values(&value), json!(["foo", "bar"]));
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::remove_empty_values;
/// use serde_json::json;
///
/// let value = json!({"foo": ["bar", null, {}, "baz"], "qux": {"adf": {}}});
///
/// assert_eq!(remove_empty_values(&value), json!({"foo": ["bar", "baz"]}));
/// ```
pub fn remove_empty_values(value: &Value) -> Value {
    match value {
        Value::Array(array) => Value::Array(
            array
                .iter()
                .map(remove_empty_values)
                .filter_map(|value| match value {
                    Value::Null => None,
                    Value::Array(array) if array.is_empty() => None,
                    Value::Object(object) if object.is_empty() => None,
                    value => Some(value),
                })
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key, remove_empty_values(value)))
                .filter_map(|(key, value)| match value {
                    Value::Null => None,
                    Value::Array(array) if array.is_empty() => None,
                    Value::Object(object) if object.is_empty() => None,
                    value => Some((key.clone(), value)),
                })
                .collect(),
        ),
        value => value.clone(),
    }
}

/// Remove one level of nesting if the data is shaped like an array.
///
/// ## Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_arrays;
/// use serde_json::json;
///
/// let value = json!([["foo"], ["bar"], [["baz"], "qux"]]);
///
/// assert_eq!(flatten_arrays(&value), json!(["foo", "bar", ["baz"], "qux"]));
/// ```
///
/// If the has only one element the array will be removed entirely, leaving the single element as
/// output.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_arrays;
/// use serde_json::json;
///
/// let value = json!(["foo"]);
///
/// assert_eq!(flatten_arrays(&value), json!("foo"));
/// ```
///
/// Non-array values will be left untouched.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_arrays;
/// use serde_json::json;
///
/// let value = json!({"foo": "bar"});
///
/// assert_eq!(flatten_arrays(&value), json!({"foo": "bar"}));
/// ```
pub fn flatten_arrays(value: &Value) -> Value {
    match value {
        Value::Array(array) if array.len() == 1 => array[0].clone(),
        Value::Array(array) => Value::Array(
            array
                .iter()
                .map(|v| match v {
                    Value::Array(a) => a.clone(),
                    _ => vec![v.clone()],
                })
                .flatten()
                .collect(),
        ),
        value => value.clone(),
    }
}

/// Flattens value to an object with flat keys.
///
/// ## Examples
///
/// Nested map with array:
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_keys;
/// use serde_json::json;
///
/// let value = json!({"foo": {"bar": ["baz", "qux"]}});
///
/// let value = flatten_keys(&value, "data");
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
/// use dts::transform::flatten_keys;
/// use serde_json::json;
///
/// let value = json!(["foo", "bar", "baz"]);
///
/// let value = flatten_keys(&value, "array");
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
/// use dts::transform::flatten_keys;
/// use serde_json::json;
///
/// let value = json!("foo");
///
/// assert_eq!(flatten_keys(&value, "data"), json!({"data": "foo"}));
/// ```
pub fn flatten_keys<P>(value: &Value, prefix: P) -> Value
where
    P: AsRef<str>,
{
    let mut flattener = KeyFlattener::new(value, prefix.as_ref());
    Value::Object(Map::from_iter(flattener.flatten().into_iter()))
}

struct KeyFlattener<'a> {
    value: &'a Value,
    prefix: &'a str,
    map: BTreeMap<String, Value>,
    stack: Vec<String>,
}

impl<'a> KeyFlattener<'a> {
    fn new(value: &'a Value, prefix: &'a str) -> Self {
        Self {
            value,
            prefix,
            map: BTreeMap::new(),
            stack: Vec::new(),
        }
    }

    fn flatten(&mut self) -> BTreeMap<String, Value> {
        self.map_value(self.value);
        self.map.clone()
    }

    fn map_value(&mut self, value: &Value) {
        match value {
            Value::Array(array) => {
                self.map.insert(self.key(), Value::Array(Vec::new()));
                for (index, value) in array.iter().enumerate() {
                    self.stack.push(FlattenKey::Index(index).to_string());
                    self.map_value(value);
                    self.stack.pop();
                }
            }
            Value::Object(object) => {
                self.map.insert(self.key(), Value::Object(Map::new()));
                for (key, value) in object.iter() {
                    self.stack.push(FlattenKey::Key(key).to_string());
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
        let acc = FlattenKey::Key(self.prefix).to_string();
        self.stack.iter().fold(acc, |mut acc, key| {
            if !acc.is_empty() && !key.starts_with('[') {
                acc.push('.');
            }
            acc.push_str(key);
            acc
        })
    }
}

enum FlattenKey<'a> {
    Index(usize),
    Key(&'a str),
}

impl<'a> ToString for FlattenKey<'a> {
    fn to_string(&self) -> String {
        match self {
            FlattenKey::Index(index) => format!("[{}]", index),
            FlattenKey::Key(key) => {
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

/// If value is of variant `Value::Object` or `Value::Array`, convert it to a `Value::String`
/// containing the json encoded string representation of the value.
pub(crate) fn collections_to_json(value: &Value) -> Value {
    if value.is_array() || value.is_object() {
        Value::String(value_to_string(value))
    } else {
        value.clone()
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

// Recursively walks `Value::Array` and `Value::Object` values and pushes all arrays and objects to
// the end of the containing `Value::Array` or `Value::Object`. This is necessary for certain
// output encodings like TOML where tables and arrays need to come after primitve values to
// disambiguate.
//
// The value is updated in place.
//
// Returns a reference to the modified value to simplify usage in iterators.
pub(crate) fn collections_to_end(value: &mut Value) -> &Value {
    if let Some(array) = value.as_array_mut() {
        let mut sortable: Vec<SortableValue> = array
            .iter_mut()
            .map(collections_to_end)
            .map(SortableValue)
            .collect();

        sortable.sort();

        *array = sortable.into_iter().map(|v| v.0.clone()).collect();
    } else if let Some(object) = value.as_object_mut() {
        let mut sortable: Vec<(&String, SortableValue)> = object
            .iter_mut()
            .map(|(k, v)| (k, collections_to_end(v)))
            .map(|(k, v)| (k, SortableValue(v)))
            .collect();

        sortable.sort_by(|a, b| a.1.cmp(&b.1));

        *object = sortable
            .into_iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect()
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_collections_to_json() {
        assert_eq!(
            collections_to_json(&json!({"foo": "bar"})),
            json!(r#"{"foo":"bar"}"#)
        );
        assert_eq!(
            collections_to_json(&json!(["foo", "bar"])),
            json!(r#"["foo","bar"]"#)
        );
        assert_eq!(collections_to_json(&json!("foo")), json!("foo"));
        assert_eq!(collections_to_json(&json!(true)), json!(true));
        assert_eq!(collections_to_json(&json!(1)), json!(1));
        assert_eq!(collections_to_json(&Value::Null), Value::Null);
    }

    #[test]
    fn test_collections_to_end() {
        assert_eq!(
            collections_to_end(
                &mut json!(["one", {"two": "three"}, [{"four": [{"five": "six"}, "seven"]}, "eight"], "nine"])
            ),
            &json!(["one", "nine", ["eight", {"four": ["seven", {"five": "six"}]}], {"two": "three"}])
        );
    }

    #[test]
    fn test_collections_to_end_object() {
        // We are comparing the JSON string representation here to assert that objects have been
        // moved to the end. Comparing the maps directly will not work as they are assumed to be
        // the same with the order ignored.
        let expected_value =
            json!({"seven": "eight", "one": {"five": "six", "two": {"three": "four"}}});
        let expected = value_to_string(&expected_value);

        let mut value = json!({"one": {"two": {"three": "four"}, "five": "six"}, "seven": "eight"});
        collections_to_end(&mut value);
        let result = value_to_string(&value);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_collections_to_end_no_change() {
        assert_eq!(
            collections_to_end(&mut json!({"foo": "bar"})),
            &json!({"foo": "bar"})
        );
        assert_eq!(
            collections_to_end(&mut json!(["foo", "bar"])),
            &json!(["foo", "bar"])
        );
        assert_eq!(collections_to_end(&mut json!("foo")), &json!("foo"));
    }
}
