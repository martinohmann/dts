//! Data transformation utilities.

use crate::{Error, Result, Value};
use jsonpath_rust::JsonPathQuery;

/// Filter value in place according to the jsonpath query.
///
/// ## Example
///
/// ```
/// use dts::transform::filter_in_place;
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
/// filter_in_place(&mut value, "$.orders[?(@.active)].id")?;
/// assert_eq!(value, json!([1, 4]));
/// #     Ok(())
/// # }
/// ```
///
/// ## Errors
///
/// This function can fail if parsing the query fails.
///
/// ```
/// use dts::transform::filter_in_place;
/// use serde_json::json;
///
/// let mut value = json!([]);
/// assert!(filter_in_place(&mut value, "$[").is_err());
/// ```
pub fn filter_in_place<Q>(value: &mut Value, query: Q) -> Result<()>
where
    Q: AsRef<str>,
{
    value
        .clone()
        .path(query.as_ref())
        .map(|filtered| *value = filtered)
        .map_err(Error::new)
}

/// Remove one level of nesting if the data is shaped like an array.
///
/// ## Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_in_place;
/// use serde_json::json;
///
/// let mut value = json!([["foo"], ["bar"], [["baz"], "qux"]]);
///
/// flatten_in_place(&mut value);
/// assert_eq!(value, json!(["foo", "bar", ["baz"], "qux"]));
/// ```
///
/// If the has only one element the array will be removed entirely, leaving the single element as
/// output.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_in_place;
/// use serde_json::json;
///
/// let mut value = json!(["foo"]);
///
/// flatten_in_place(&mut value);
/// assert_eq!(value, json!("foo"));
/// ```
///
/// Non-array values will be left untouched.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts::transform::flatten_in_place;
/// use serde_json::json;
///
/// let mut value = json!({"foo": "bar"});
///
/// flatten_in_place(&mut value);
/// assert_eq!(value, json!({"foo": "bar"}));
/// ```
pub fn flatten_in_place(value: &mut Value) {
    if let Some(array) = value.as_array() {
        *value = if array.len() == 1 {
            array[0].clone()
        } else {
            Value::Array(
                array
                    .iter()
                    .map(|v| match v {
                        Value::Array(a) => a.clone(),
                        _ => vec![v.clone()],
                    })
                    .flatten()
                    .collect(),
            )
        };
    }
}

/// If value is of variant `Value::Object` or `Value::Array`, convert it to a `Value::String`
/// containing the json encoded string representation of the value.
pub(crate) fn collections_to_json(value: &Value) -> Result<Value> {
    match value {
        Value::Array(value) => Ok(Value::String(serde_json::to_string(value)?)),
        Value::Object(value) => Ok(Value::String(serde_json::to_string(value)?)),
        _ => Ok(value.clone()),
    }
}

#[derive(PartialEq, Eq)]
struct SortableValue<'a>(&'a Value);

use std::cmp::Ordering;

impl<'a> PartialOrd for SortableValue<'a> {
    fn partial_cmp(&self, other: &SortableValue) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for SortableValue<'a> {
    fn cmp(&self, other: &SortableValue) -> Ordering {
        use serde_json::Value::*;

        // Sort order: primitives, arrays, objects. Original order is preserved as much as possible
        // by avoiding to compare the values wrapped by each variant directly.
        match (self, other) {
            (SortableValue(Array(_)), SortableValue(Array(_))) => Ordering::Equal,
            (SortableValue(Array(_)), SortableValue(Object(_))) => Ordering::Less,
            (SortableValue(Array(_)), _) => Ordering::Greater,
            (SortableValue(Object(_)), SortableValue(Object(_))) => Ordering::Equal,
            (SortableValue(Object(_)), SortableValue(Array(_))) => Ordering::Greater,
            (SortableValue(Object(_)), _) => Ordering::Greater,
            (_, SortableValue(Array(_))) => Ordering::Less,
            (_, SortableValue(Object(_))) => Ordering::Less,
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
pub(crate) fn collections_to_end<'a>(value: &'a mut Value) -> &'a Value {
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
            collections_to_json(&json!({"foo": "bar"})).unwrap(),
            json!(r#"{"foo":"bar"}"#)
        );
        assert_eq!(
            collections_to_json(&json!(["foo", "bar"])).unwrap(),
            json!(r#"["foo","bar"]"#)
        );
        assert_eq!(collections_to_json(&json!("foo")).unwrap(), json!("foo"));
        assert_eq!(collections_to_json(&json!(true)).unwrap(), json!(true));
        assert_eq!(collections_to_json(&json!(1)).unwrap(), json!(1));
        assert_eq!(collections_to_json(&Value::Null).unwrap(), Value::Null);
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
        let expected = serde_json::to_string(&expected_value).unwrap();

        let mut value = json!({"one": {"two": {"three": "four"}, "five": "six"}, "seven": "eight"});
        collections_to_end(&mut value);
        let result = serde_json::to_string(&value).unwrap();

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
