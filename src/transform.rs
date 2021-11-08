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
}
