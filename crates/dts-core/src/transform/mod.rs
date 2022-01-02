//! Data transformation utilities.

pub mod definition;
mod error;
pub(crate) mod key;
pub(crate) mod sort;

pub use definition::{Arg, Definition, DefinitionMatch, Definitions};
pub use error::*;

use crate::parsers::flat_key::{self, KeyPart, KeyParts};
use crate::Result;
use dts_json::{Map, Value};
use jsonpath_rust::JsonPathQuery;
use key::KeyFlattener;
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value as JsonValue;
use sort::ValueSorter;
use std::iter;
use std::str::FromStr;

/// A type that can apply transformations to a `Value`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Transformation {
    /// Remove one level of nesting if the data is shaped like an array or one-elemented object.
    Flatten,
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
    /// Extracts object keys.
    Keys,
    /// Delete object keys matching a pattern.
    DeleteKeys(String),
    /// Sort objects and arrays.
    Sort(ValueSorter),
    /// Convert all arrays into objects.
    ArraysToObjects,
}

impl Transformation {
    /// Applies the `Transformation` to a value.
    ///
    /// ## Errors
    ///
    /// If the `Transformation::JsonPath` variant is applied with a malformed query `apply_chain`
    /// returns a `TransformError`.
    pub fn apply(&self, value: Value) -> Result<Value, TransformError> {
        let value = match self {
            Self::Flatten => flatten(value),
            Self::FlattenKeys(prefix) => {
                flatten_keys(value, prefix.as_ref().unwrap_or(&String::from("data")))
            }
            Self::JsonPath(query) => filter_jsonpath(value, query)?,
            Self::RemoveEmptyValues => remove_empty_values(value),
            Self::Chain(chain) => apply_chain(chain, value)?,
            Self::DeepMerge => deep_merge(value),
            Self::ExpandKeys => expand_keys(value),
            Self::Keys => keys(value),
            Self::DeleteKeys(pattern) => delete_keys(value, pattern)?,
            Self::Sort(sorter) => sort(sorter, value),
            Self::ArraysToObjects => arrays_to_objects(value),
        };

        Ok(value)
    }
}

impl FromStr for Transformation {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let transformation = if s.contains(',') {
            let chain = s
                .split(',')
                .map(Self::from_str)
                .collect::<Result<Vec<_>, _>>()?;

            Self::Chain(chain)
        } else {
            let (key, value) = match s.find('=') {
                Some(pos) => (&s[..pos], Some(&s[pos + 1..])),
                None => (s, None),
            };

            match key {
                "f" | "flatten" => Self::Flatten,
                "F" | "flatten-keys" => Self::FlattenKeys(value.map(|v| v.to_string())),
                "j" | "jsonpath" => value
                    .map(|query| Self::JsonPath(query.to_string()))
                    .ok_or_else(|| TransformError::value_required(key))?,
                "r" | "remove-empty-values" => Self::RemoveEmptyValues,
                "m" | "deep-merge" => Self::DeepMerge,
                "e" | "expand-keys" => Self::ExpandKeys,
                "k" | "keys" => Self::Keys,
                "d" | "delete-keys" => value
                    .map(|pattern| Self::DeleteKeys(pattern.to_string()))
                    .ok_or_else(|| TransformError::value_required(key))?,
                "s" | "sort" => {
                    let sorter = match value {
                        Some(value) => ValueSorter::from_str(value)?,
                        None => ValueSorter::default(),
                    };

                    Self::Sort(sorter)
                }
                "ato" | "arrays-to-objects" => Self::ArraysToObjects,
                key => return Err(TransformError::unknown_transformation(key)),
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
pub fn apply_chain<'a, I>(chain: I, value: Value) -> Result<Value, TransformError>
where
    I: IntoIterator<Item = &'a Transformation>,
{
    chain
        .into_iter()
        .try_fold(value, |value, transformation| transformation.apply(value))
}

/// Filter value according to the jsonpath query.
///
/// ## Example
///
/// ```
/// use dts_core::transform::filter_jsonpath;
/// use dts_json::json;
/// # use pretty_assertions::assert_eq;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!({
///   "orders": [
///     {"id": 1, "active": true},
///     {"id": 2},
///     {"id": 3},
///     {"id": 4, "active": true}
///   ]
/// });
///
/// assert_eq!(filter_jsonpath(value, "$.orders[?(@.active)].id")?, json!([1, 4]));
/// #     Ok(())
/// # }
/// ```
///
/// ## Errors
///
/// This function can fail if parsing the query fails.
///
/// ```
/// use dts_core::transform::filter_jsonpath;
/// use dts_json::json;
///
/// let value = json!([]);
/// assert!(filter_jsonpath(value, "$[").is_err());
/// ```
pub fn filter_jsonpath<Q>(value: Value, query: Q) -> Result<Value, TransformError>
where
    Q: AsRef<str>,
{
    JsonValue::from(value)
        .path(query.as_ref())
        .map(Into::into)
        .map_err(TransformError::JSONPathParseError)
}

/// Removes nulls, empty arrays and empty objects from value. Top level empty values are not
/// removed.
///
/// ## Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::remove_empty_values;
/// use dts_json::{json, Value};
///
/// let value = Value::Null;
///
/// assert_eq!(remove_empty_values(value), Value::Null);
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::remove_empty_values;
/// use dts_json::{json, Value};
///
/// let mut value = json!({});
///
/// assert_eq!(remove_empty_values(value), json!({}));
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::remove_empty_values;
/// use dts_json::{json, Value};
///
/// let value = json!(["foo", null, "bar"]);
///
/// assert_eq!(remove_empty_values(value), json!(["foo", "bar"]));
/// ```
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::remove_empty_values;
/// use dts_json::json;
///
/// let value = json!({"foo": ["bar", null, {}, "baz"], "qux": {"adf": {}}});
///
/// assert_eq!(remove_empty_values(value), json!({"foo": ["bar", "baz"]}));
/// ```
pub fn remove_empty_values(value: Value) -> Value {
    match value {
        Value::Array(array) => Value::Array(
            array
                .into_iter()
                .map(remove_empty_values)
                .filter(|value| !value.is_empty())
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| (key, remove_empty_values(value)))
                .filter(|(_, value)| !value.is_empty())
                .collect(),
        ),
        value => value,
    }
}

/// Removes one level of nesting if the data is shaped like an array or one-elemented object.
///
/// ## Examples
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::flatten;
/// use dts_json::json;
///
/// let value = json!([["foo"], ["bar"], [["baz"], "qux"]]);
///
/// assert_eq!(flatten(value), json!(["foo", "bar", ["baz"], "qux"]));
/// ```
///
/// If the has only one element the array will be removed entirely, leaving the single element as
/// output.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::flatten;
/// use dts_json::json;
///
/// let value = json!(["foo"]);
///
/// assert_eq!(flatten(value), json!("foo"));
/// ```
///
/// One-element objects will be flattened to their value.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::flatten;
/// use dts_json::json;
///
/// let value = json!({"foo": "bar"});
///
/// assert_eq!(flatten(value), json!("bar"));
/// ```
///
/// Objects with more that one key will be left untouched.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::flatten;
/// use dts_json::json;
///
/// let value = json!({"foo": "bar", "baz": "qux"});
///
/// assert_eq!(flatten(value), json!({"foo": "bar", "baz": "qux"}));
/// ```
pub fn flatten(value: Value) -> Value {
    match value {
        Value::Array(mut array) if array.len() == 1 => array.swap_remove(0),
        Value::Array(array) => {
            Value::Array(array.into_iter().flat_map(Value::into_array).collect())
        }
        Value::Object(object) if object.len() == 1 => {
            object.into_iter().next().map(|(_, v)| v).unwrap()
        }
        value => value,
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
/// use dts_core::transform::flatten_keys;
/// use dts_json::json;
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
/// use dts_core::transform::flatten_keys;
/// use dts_json::json;
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
/// use dts_core::transform::flatten_keys;
/// use dts_json::json;
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
/// use dts_core::transform::expand_keys;
/// use dts_json::json;
///
/// let value = json!([{"foo.bar": 1, "foo[\"bar-baz\"]": 2}]);
/// let expected = json!([{"foo": {"bar": 1, "bar-baz": 2}}]);
///
/// assert_eq!(expand_keys(value), expected);
/// ```
pub fn expand_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => object
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

/// Recursively merges all arrays and maps in `value`. If `value` is not an array it is returned
/// as is.
pub fn deep_merge(value: Value) -> Value {
    match value {
        Value::Array(mut array) => {
            array
                .iter_mut()
                .fold(Value::Array(Vec::new()), |mut acc, value| {
                    acc.deep_merge(value);
                    acc
                })
        }
        value => value,
    }
}

/// Extracts object keys into a new `Value`. The returned `Value` is always of variant
/// `Value::Array`. If the input is not a `Value::Object`, the returned array is empty.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::keys;
/// use dts_json::json;
///
/// let value = json!({"foo": "bar", "baz": "qux"});
///
/// assert_eq!(keys(value), json!(["foo", "baz"]));
/// ```
pub fn keys(value: Value) -> Value {
    Value::Array(
        value
            .as_object()
            .map(|obj| obj.keys().cloned().map(Value::String).collect())
            .unwrap_or_default(),
    )
}

/// Recursively deletes all keys matching the regex pattern.
///
/// ```
/// use dts_core::transform::delete_keys;
/// use dts_json::json;
/// use regex::Regex;
/// # use pretty_assertions::assert_eq;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!({"foo": "bar", "baz": {"foobar": "qux", "one": "two"}});
///
/// assert_eq!(delete_keys(value, "^fo")?, json!({"baz": {"one": "two"}}));
/// #   Ok(())
/// # }
/// ```
pub fn delete_keys(value: Value, pattern: &str) -> Result<Value, TransformError> {
    let regex = Regex::new(pattern)?;
    Ok(delete_keys_impl(value, &regex))
}

fn delete_keys_impl(value: Value, regex: &Regex) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter(|(key, _)| !regex.is_match(key))
                .map(|(key, value)| (key, delete_keys_impl(value, regex)))
                .collect(),
        ),
        Value::Array(array) => Value::Array(
            array
                .into_iter()
                .map(|value| delete_keys_impl(value, regex))
                .collect(),
        ),
        value => value,
    }
}

/// Recursively sorts all maps and arrays.
pub fn sort(sorter: &ValueSorter, mut value: Value) -> Value {
    sorter.sort(&mut value);
    value
}

/// Recursively transforms all arrays into objects with the array index as key.
pub fn arrays_to_objects(value: Value) -> Value {
    match value {
        Value::Array(array) => Value::Object(
            array
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), arrays_to_objects(v)))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(k, v)| (k, arrays_to_objects(v)))
                .collect(),
        ),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dts_json::json;
    use pretty_assertions::assert_eq;

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
        assert_eq!(Transformation::from_str("flatten").unwrap(), Flatten);
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
            Transformation::from_str("F=prefix,r,flatten,r,jsonpath=$").unwrap(),
            Transformation::Chain(vec![
                FlattenKeys(Some("prefix".into())),
                RemoveEmptyValues,
                Flatten,
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
            Flatten,
        ];

        assert_eq!(
            apply_chain(&transformations, json!([null, "foo", {"bar": "baz"}])).unwrap(),
            json!("baz")
        );
    }

    #[test]
    fn test_deep_merge() {
        assert_eq!(deep_merge(Value::Null), Value::Null);
        assert_eq!(deep_merge(json!("a")), json!("a"));
        assert_eq!(deep_merge(json!([1, null, "three"])), json!("three"));
        assert_eq!(
            deep_merge(json!([[1, "two"], ["three", 4]])),
            json!(["three", 4])
        );
        assert_eq!(
            deep_merge(json!([[null, 1, "two"], ["three", 4]])),
            json!(["three", 4, "two"])
        );
        assert_eq!(
            deep_merge(json!([[1, "two"], [null, null, 4]])),
            json!([1, "two", 4])
        );
        assert_eq!(
            deep_merge(json!([{"foo": "bar"},
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
            expand_keys(value),
            json!({"data": {"foo": {"bar": ["baz", "qux"]}}})
        );
    }

    #[test]
    fn test_arrays_to_objects() {
        assert_eq!(
            arrays_to_objects(json!([{"foo": "bar"},{"bar": [1], "qux": null}])),
            json!({"0": {"foo": "bar"}, "1": {"bar": {"0": 1}, "qux": null}})
        );
    }
}
