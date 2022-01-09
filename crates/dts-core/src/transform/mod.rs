//! Data transformation utilities.

pub mod dsl;
pub mod jsonpath;
pub(crate) mod key;
pub mod sort;

use crate::parsers::flat_key::{self, KeyPart, KeyParts};
use dts_json::{Map, Value};
use jsonpath::{JsonPathMutator, JsonPathSelector};
use key::KeyFlattener;
use rayon::prelude::*;
use regex::Regex;
use sort::ValueSorter;
use std::fmt::Debug;
use std::iter;

/// Represents a thing that can take a value, transform it and produce a new value.
pub trait Transform {
    /// Takes a `Value`, applies a transformation and yields a new `Value`.
    fn transform(&self, value: Value) -> Value;
}

impl<T> Transform for Box<T>
where
    T: Transform + ?Sized,
{
    fn transform(&self, value: Value) -> Value {
        (**self).transform(value)
    }
}

impl<T> Transform for &T
where
    T: Transform + ?Sized,
{
    fn transform(&self, value: Value) -> Value {
        (*self).transform(value)
    }
}

/// Represents a chain of transformation operations.
pub struct Chain {
    inner: Vec<Box<dyn Transform>>,
}

impl Chain {
    /// Creates a new `Chain` for an iterator.
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Transform>>,
    {
        Chain {
            inner: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<Box<dyn Transform>> for Chain {
    fn from_iter<I: IntoIterator<Item = Box<dyn Transform>>>(iter: I) -> Self {
        Chain::new(iter)
    }
}

impl IntoIterator for Chain {
    type Item = Box<dyn Transform>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl Transform for Chain {
    fn transform(&self, value: Value) -> Value {
        self.inner
            .iter()
            .fold(value, |value, trans| trans.transform(value))
    }
}

/// A type that can select a value based on a jsonpath query.
pub struct Select(JsonPathSelector);

impl Select {
    /// Creates a new `Mutate`.
    pub fn new(selector: JsonPathSelector) -> Self {
        Select(selector)
    }
}

impl Transform for Select {
    fn transform(&self, value: Value) -> Value {
        self.0.select(value)
    }
}

/// A type that can selectively mutate a value based on a jsonpath query and a chain of
/// transformations.
pub struct Mutate {
    mutator: JsonPathMutator,
    chain: Chain,
}

impl Mutate {
    /// Creates a new `Mutate`.
    pub fn new(mutator: JsonPathMutator, chain: Chain) -> Self {
        Mutate { mutator, chain }
    }
}

impl Transform for Mutate {
    fn transform(&self, value: Value) -> Value {
        self.mutator
            .mutate(value, |v| Some(self.chain.transform(v)))
    }
}

/// A type that can selectively delete values based on a jsonpath query. Deleted values are
/// represented as `Value::Null`.
pub struct Delete(JsonPathMutator);

impl Delete {
    /// Creates a new `Delete`.
    pub fn new(mutator: JsonPathMutator) -> Self {
        Delete(mutator)
    }
}

impl Transform for Delete {
    fn transform(&self, value: Value) -> Value {
        self.0.mutate(value, |_| Some(Value::Null))
    }
}

/// A type that can selectively remove values based on a jsonpath query.
pub struct Remove(JsonPathMutator);

impl Remove {
    /// Creates a new `Remove`.
    pub fn new(mutator: JsonPathMutator) -> Self {
        Remove(mutator)
    }
}

impl Transform for Remove {
    fn transform(&self, value: Value) -> Value {
        self.0.mutate(value, |_| None)
    }
}

/// Flattens value to an object with flat keys.
pub struct FlattenKeys(String);

impl FlattenKeys {
    /// Creates a new `FlattenKeys`.
    pub fn new(prefix: &str) -> Self {
        FlattenKeys(prefix.to_owned())
    }
}

impl Transform for FlattenKeys {
    fn transform(&self, value: Value) -> Value {
        flatten_keys(value, &self.0)
    }
}

/// Deletes object keys matching a pattern.
pub struct DeleteKeys(Regex);

impl DeleteKeys {
    /// Creates a new `DeleteKeys`.
    pub fn new(regex: Regex) -> Self {
        DeleteKeys(regex)
    }
}

impl Transform for DeleteKeys {
    fn transform(&self, value: Value) -> Value {
        delete_keys(value, &self.0)
    }
}

/// Sorts objects and arrays.
pub struct Sort(ValueSorter);

impl Sort {
    /// Creates a new `Sort`.
    pub fn new(sorter: ValueSorter) -> Self {
        Sort(sorter)
    }
}

impl Transform for Sort {
    fn transform(&self, value: Value) -> Value {
        sort(&self.0, value)
    }
}

/// EachKey applies a chain of transformations to every key of an object. For values of any other
/// type this is a no-op.
pub struct EachKey(Chain);

impl EachKey {
    /// Creates a new `EachKey` which applies the `Chain` to every key of an object.
    pub fn new(chain: Chain) -> Self {
        EachKey(chain)
    }
}

impl Transform for EachKey {
    fn transform(&self, value: Value) -> Value {
        match value {
            Value::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(key, value)| (self.0.transform(key.into()).into_string(), value))
                    .collect(),
            ),
            value => value,
        }
    }
}

/// EachValue applies a chain of transformations to every value of an array or object. For values
/// of any other type this is a no-op.
pub struct EachValue(Chain);

impl EachValue {
    /// Creates a new `EachValue` which applies the `Chain` to every value of an array or object.
    pub fn new(chain: Chain) -> Self {
        EachValue(chain)
    }
}

impl Transform for EachValue {
    fn transform(&self, value: Value) -> Value {
        match value {
            Value::Array(array) => Value::Array(
                array
                    .into_iter()
                    .map(|value| self.0.transform(value))
                    .collect(),
            ),
            Value::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(key, value)| (key, self.0.transform(value)))
                    .collect(),
            ),
            value => value,
        }
    }
}

/// A type that can apply unparameterized transformations to a `Value`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Unparameterized {
    /// Remove one level of nesting if the data is shaped like an array or one-elemented object.
    Flatten,
    /// Removes nulls, empty arrays and empty objects from value. Top level empty values are not
    /// removed.
    RemoveEmptyValues,
    /// Deep merge values if the top level value is an array.
    DeepMerge,
    /// Expands flat keys to nested objects.
    ExpandKeys,
    /// Extracts object keys.
    Keys,
    /// Convert all arrays into objects.
    ArraysToObjects,
    /// Extracts array and object values.
    Values,
}

impl Transform for Unparameterized {
    fn transform(&self, value: Value) -> Value {
        match self {
            Self::Flatten => flatten(value),
            Self::RemoveEmptyValues => remove_empty_values(value),
            Self::DeepMerge => deep_merge(value),
            Self::ExpandKeys => expand_keys(value),
            Self::Keys => keys(value),
            Self::ArraysToObjects => arrays_to_objects(value),
            Self::Values => values(value),
        }
    }
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

/// Extracts array and object values into a new `Value`. The returned `Value` is always of variant
/// `Value::Array`. If the input is not a `Value::Array` or `Value::Object`, the returned array is
/// empty.
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use dts_core::transform::values;
/// use dts_json::json;
///
/// let value = json!({"foo": "bar", "baz": "qux"});
///
/// assert_eq!(values(value), json!(["bar", "qux"]));
/// ```
pub fn values(value: Value) -> Value {
    match value {
        Value::Array(array) => Value::Array(array),
        Value::Object(object) => Value::Array(object.into_iter().map(|(_, v)| v).collect()),
        _ => Value::Array(vec![]),
    }
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
/// let regex = Regex::new("^fo")?;
///
/// assert_eq!(delete_keys(value, &regex), json!({"baz": {"one": "two"}}));
/// #   Ok(())
/// # }
/// ```
pub fn delete_keys(value: Value, regex: &Regex) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter(|(key, _)| !regex.is_match(key))
                .map(|(key, value)| (key, delete_keys(value, regex)))
                .collect(),
        ),
        Value::Array(array) => Value::Array(
            array
                .into_iter()
                .map(|value| delete_keys(value, regex))
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
    fn test_chain() {
        use Unparameterized::*;

        let transformations: Vec<Box<dyn Transform>> = vec![
            Box::new(FlattenKeys::new("data")),
            Box::new(RemoveEmptyValues),
            Box::new(Select(JsonPathSelector::new("$['data[2].bar']").unwrap())),
            Box::new(Flatten),
        ];

        let chain = Chain::from_iter(transformations);

        assert_eq!(
            chain.transform(json!([null, "foo", {"bar": "baz"}])),
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
