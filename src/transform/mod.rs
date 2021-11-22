//! Data transformation utilities.

pub(crate) mod key;

use crate::parsers::flat_key::{KeyPart, KeyParts};
use crate::{Error, Result, Value};
use jsonpath_rust::JsonPathQuery;
use key::KeyFlattener;
use serde_json::Map;
use std::str::FromStr;

/// A type that can apply transformations to a `Value`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Transformation {
    /// Remove one level of nesting if the data is shaped like an array.
    FlattenArrays,
    /// Flattens value to an object with flat keys.
    FlattenKeys(Option<String>),
    /// Filter value according to a jsonpath query.
    JsonPath(String),
    /// Removes nulls, empty arrays and empty objects from value. Top level empty values are not
    /// removed.
    RemoveEmptyValues,
    /// A chain of multiple transformations.
    Chain(Vec<Transformation>),
    /// Deep merge values if the top level value is an array.
    DeepMerge,
    /// Expands flat keys to nested objects.
    ExpandKeys,
}

impl Transformation {
    /// Applies the `Transformation` to a value.
    ///
    /// ## Errors
    ///
    /// If the `Transformation::JsonPath` variant is applied with a malformed query `apply_chain`
    /// returns an `Error`.
    pub fn apply(&self, value: &Value) -> Result<Value> {
        let value = match self {
            Self::FlattenArrays => flatten_arrays(value),
            Self::FlattenKeys(prefix) => flatten_keys(
                value,
                prefix.clone().unwrap_or_else(|| String::from("data")),
            ),
            Self::JsonPath(query) => filter_jsonpath(value, query)?,
            Self::RemoveEmptyValues => remove_empty_values(value),
            Self::Chain(chain) => apply_chain(chain, value)?,
            Self::DeepMerge => deep_merge(value),
            Self::ExpandKeys => expand_keys(value)?,
        };

        Ok(value)
    }
}

impl FromStr for Transformation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let transformation = if s.contains(',') {
            let chain = s
                .split(',')
                .map(Self::from_str)
                .collect::<Result<Vec<_>>>()?;

            Self::Chain(chain)
        } else {
            let (key, value) = match s.find('=') {
                Some(pos) => (&s[..pos], Some(&s[pos + 1..])),
                None => (s, None),
            };

            match key {
                "f" | "flatten-arrays" => Self::FlattenArrays,
                "F" | "flatten-keys" => Self::FlattenKeys(value.map(|v| v.to_string())),
                "j" | "jsonpath" => match value {
                    Some(query) => Self::JsonPath(query.to_string()),
                    None => return Err(Error::new("jsonpath expects a filter query")),
                },
                "r" | "remove-empty-values" => Self::RemoveEmptyValues,
                "m" | "deep-merge" => Self::DeepMerge,
                "e" | "expand-keys" => Self::ExpandKeys,
                key => return Err(Error::new(format!("unknown transformation: {}", key))),
            }
        };

        Ok(transformation)
    }
}

/// Applies a chain of transformations to a value.
///
/// ## Errors
///
/// If the `Transformation::JsonPath` variant is applied with a malformed query `apply_chain`
/// returns an `Error`.
pub fn apply_chain<'a, I>(chain: I, value: &Value) -> Result<Value>
where
    I: IntoIterator<Item = &'a Transformation>,
{
    chain
        .into_iter()
        .try_fold(value.clone(), |value, transformation| {
            transformation.apply(&value)
        })
}

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
    value.clone().path(query.as_ref()).map_err(Error::JsonPath)
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

/// Recursively expands flat keys to nested objects.
pub fn expand_keys(value: &Value) -> Result<Value> {
    match value {
        Value::Object(object) => object.iter().try_fold(Value::Null, |acc, (key, value)| {
            let mut parts = KeyParts::parse(key)?;
            parts.reverse();
            let value = expand_key_parts(&mut parts, value);
            Ok(deep_merge_values(&acc, &value))
        }),
        Value::Array(array) => Ok(Value::Array(
            array.iter().map(expand_keys).collect::<Result<_>>()?,
        )),
        value => Ok(value.clone()),
    }
}

fn expand_key_parts(parts: &mut KeyParts, value: &Value) -> Value {
    match parts.pop() {
        Some(key) => match key {
            KeyPart::Ident(ident) => {
                let mut object = Map::new();
                object.insert(ident, expand_key_parts(parts, value));
                Value::Object(object)
            }
            KeyPart::Index(index) => {
                let mut array = vec![Value::Null; index + 1];
                array[index] = expand_key_parts(parts, value);
                Value::Array(array)
            }
        },
        None => value.clone(),
    }
}

/// Recursively merges all arrays and maps in `value`. If `value` is not an array it is returned
/// as is.
pub fn deep_merge(value: &Value) -> Value {
    match value.as_array() {
        Some(array) => array.iter().fold(Value::Array(Vec::new()), |acc, value| {
            deep_merge_values(&acc, value)
        }),
        None => value.clone(),
    }
}

fn deep_merge_values(lhs: &Value, rhs: &Value) -> Value {
    match (lhs, rhs) {
        (Value::Object(lhs), Value::Object(rhs)) => {
            let mut merged = lhs.clone();

            for (key, value) in rhs.iter() {
                merged
                    .entry(key)
                    .and_modify(|merged| *merged = deep_merge_values(merged, value))
                    .or_insert_with(|| value.clone());
            }

            Value::Object(merged)
        }
        (Value::Array(lhs), Value::Array(rhs)) => {
            let mut merged = lhs.clone();

            merged.resize(lhs.len().max(rhs.len()), Value::Null);

            for (i, value) in rhs.iter().enumerate() {
                merged[i] = deep_merge_values(&merged[i], value);
            }

            Value::Array(merged)
        }
        (_, Value::Null) => lhs.clone(),
        (_, _) => rhs.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_transformation_from_str() {
        use Transformation::*;

        assert_eq!(
            Transformation::from_str("j=.").unwrap(),
            JsonPath(".".into())
        );
        assert_eq!(
            Transformation::from_str("j=$[*]").unwrap(),
            JsonPath("$[*]".into())
        );
        assert_eq!(
            Transformation::from_str("flatten-arrays").unwrap(),
            FlattenArrays
        );
        assert_eq!(Transformation::from_str("F").unwrap(), FlattenKeys(None));
        assert_eq!(
            Transformation::from_str("flatten-keys").unwrap(),
            FlattenKeys(None)
        );
        assert_eq!(
            Transformation::from_str("F=json").unwrap(),
            FlattenKeys(Some("json".into()))
        );
        assert_eq!(
            Transformation::from_str("flatten-keys=foo").unwrap(),
            FlattenKeys(Some("foo".into()))
        );
        assert_eq!(Transformation::from_str("r").unwrap(), RemoveEmptyValues);
        assert_eq!(
            Transformation::from_str("remove-empty-values").unwrap(),
            RemoveEmptyValues
        );
    }

    #[test]
    fn test_transformation_chain_from_str() {
        use Transformation::*;

        assert_eq!(
            Transformation::from_str("F=prefix,r,flatten-arrays,r,jsonpath=$").unwrap(),
            Transformation::Chain(vec![
                FlattenKeys(Some("prefix".into())),
                RemoveEmptyValues,
                FlattenArrays,
                RemoveEmptyValues,
                JsonPath("$".into()),
            ])
        );
    }

    #[test]
    fn test_transformation_from_str_errors() {
        assert!(Transformation::from_str("j").is_err());
        assert!(Transformation::from_str("jsonpath").is_err());
        assert!(Transformation::from_str("f,r,baz").is_err());
    }

    #[test]
    fn test_apply_chain() {
        use Transformation::*;

        let transformations = vec![
            FlattenKeys(None),
            RemoveEmptyValues,
            JsonPath("$['data[2].bar']".into()),
            FlattenArrays,
        ];

        assert_eq!(
            apply_chain(&transformations, &json!([null, "foo", {"bar": "baz"}])).unwrap(),
            json!("baz")
        );
    }

    #[test]
    fn test_deep_merge() {
        assert_eq!(deep_merge(&Value::Null), Value::Null);
        assert_eq!(deep_merge(&json!("a")), json!("a"));
        assert_eq!(deep_merge(&json!([1, null, "three"])), json!("three"));
        assert_eq!(
            deep_merge(&json!([[1, "two"], ["three", 4]])),
            json!(["three", 4])
        );
        assert_eq!(
            deep_merge(&json!([[null, 1, "two"], ["three", 4]])),
            json!(["three", 4, "two"])
        );
        assert_eq!(
            deep_merge(&json!([[1, "two"], [null, null, 4]])),
            json!([1, "two", 4])
        );
        assert_eq!(
            deep_merge(&json!([{"foo": "bar"},
                       {"foo": {"bar": "baz"}, "bar": [1], "qux": null},
                       {"foo": {"bar": "qux"}, "bar": [2], "baz": 1}])),
            json!({"foo": {"bar": "qux"}, "bar": [2], "baz": 1, "qux": null})
        );
    }

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
            expand_keys(&value).unwrap(),
            json!({"data": {"foo": {"bar": ["baz", "qux"]}}})
        );
    }
}
