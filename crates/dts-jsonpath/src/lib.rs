#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
pub mod path;

pub use crate::error::{Error, Result};
use crate::parser::ast::Selector;
pub use crate::parser::parse;
pub use crate::path::compile;
use dts_json::Value;
use std::str::FromStr;

/// Represents a jsonpath query that can be used for filtering and mutating json values.
#[derive(Debug, Clone)]
pub struct JsonPath {
    selectors: Vec<Selector>,
}

impl JsonPath {
    /// Creates a new `JsonPath` from a query. The returned value can be used multiple times.
    ///
    /// ## Errors
    ///
    /// Returns an error if the input is not a valid jsonpath query.
    pub fn new(query: &str) -> Result<JsonPath> {
        let selectors = parse(query)?;
        Ok(JsonPath { selectors })
    }

    /// Selects all matching `Value`s in `root` and returns references to them.
    ///
    /// See [`find`] if you prefer an API that works with owned `Value`s.
    ///
    /// [`find`]: JsonPath::find
    pub fn select<'a>(&'a self, root: &'a Value) -> Vec<&'a Value> {
        compile(&self.selectors, root).select(root)
    }

    /// Selects all matching `Value`s from `root` and returns copies of them.
    ///
    /// See [`select`] if you are looking to select values without cloning.
    ///
    /// [`select`]: JsonPath::select
    ///
    /// ```
    /// use dts_json::json;
    /// use dts_jsonpath::JsonPath;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({
    ///     "books": [
    ///         {
    ///             "title": "Sayings of the Century",
    ///             "price": 8.95
    ///         },
    ///         {
    ///             "title": "Sword of Honour",
    ///             "price": 12.99
    ///         },
    ///         {
    ///             "title": "Moby Dick",
    ///             "price": 8.99
    ///         },
    ///         {
    ///             "title": "The Lord of the Rings",
    ///             "price": 22.99
    ///         }
    ///     ]
    /// });
    ///
    /// let path = JsonPath::new("$.books[?(@.price < 10)].title")?;
    ///
    /// assert_eq!(path.find(value), json!([
    ///     "Sayings of the Century",
    ///     "Moby Dick",
    /// ]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn find(&self, root: Value) -> Value {
        self.select(&root).clone().into()
    }

    /// Recursively visits `root` and calls `f` for every matching `Value` to mutate it in-place.
    ///
    /// See [`mutate`] if you prefer an API that works with owned `Value`s.
    ///
    /// [`mutate`]: JsonPath::mutate
    pub fn visit<F>(&self, root: &mut Value, f: F)
    where
        F: FnMut(&mut Value),
    {
        let root_ref = root.clone();
        compile(&self.selectors, &root_ref).visit(root, f);
    }

    /// Recursively visits `root` and calls `f` for every matching `Value`, producing a new
    /// `Value`.
    ///
    /// See [`visit`] if you are looking for mutating the matched values in-place without cloning.
    ///
    /// [`visit`]: JsonPath::visit
    ///
    /// ```
    /// use dts_json::{json, Value};
    /// use dts_jsonpath::JsonPath;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let root = json!({
    ///     "books": [
    ///         {
    ///             "title": "Sayings of the Century",
    ///             "price": 8.95
    ///         },
    ///         {
    ///             "title": "Sword of Honour",
    ///             "price": 12.99
    ///         },
    ///         {
    ///             "title": "Moby Dick",
    ///             "price": 8.99
    ///         },
    ///         {
    ///             "title": "The Lord of the Rings",
    ///             "price": 22.99
    ///         }
    ///     ]
    /// });
    ///
    /// let path = JsonPath::new("$.books[?(@.price > 10)].price")?;
    ///
    /// let mutated = path.mutate(root, |value| {
    ///     match value {
    ///         // Give a discount!
    ///         Value::Number(num) => num.as_f64().map(|price| price - 4.0).unwrap_or(0.0).into(),
    ///         value => value,
    ///     }
    /// });
    ///
    /// assert_eq!(mutated, json!({
    ///     "books": [
    ///         {
    ///             "title": "Sayings of the Century",
    ///             "price": 8.95
    ///         },
    ///         {
    ///             "title": "Sword of Honour",
    ///             "price": 8.99
    ///         },
    ///         {
    ///             "title": "Moby Dick",
    ///             "price": 8.99
    ///         },
    ///         {
    ///             "title": "The Lord of the Rings",
    ///             "price": 18.99
    ///         }
    ///     ]
    /// }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn mutate<F>(&self, mut root: Value, f: F) -> Value
    where
        F: Fn(Value) -> Value,
    {
        self.visit(&mut root, |value| *value = f(value.clone()));
        root
    }

    /// Recursively visits `root` and replaces all matches with the `Value` returned by `f`.
    ///
    /// This is similar to [`replace`] but accepts a closure instead of a `Value`.
    ///
    /// [`replace`]: JsonPath::replace
    pub fn replace_with<F>(&self, value: Value, f: F) -> Value
    where
        F: Fn() -> Value,
    {
        self.mutate(value, |_| f())
    }

    /// Recursively visits `root` and replaces all matches with the `replacement`.
    ///
    /// This is similar to [`replace_with`] but accepts a `Value` instead of a closure.
    ///
    /// [`replace_with`]: JsonPath::replace_with
    pub fn replace(&self, value: Value, replacement: Value) -> Value {
        self.replace_with(value, || replacement.clone())
    }
}

impl FromStr for JsonPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPath::new(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[test]
    fn test_find() {
        let path = JsonPath::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        assert_eq!(path.find(value), json!([2, 3]));
    }

    #[test]
    fn test_replace_with() {
        let path = JsonPath::new("$.foo.*").unwrap();

        let value = json!({"bar": {"baz": 1}, "foo": {"bar": 2, "qux": 3}});
        let result = path.replace_with(value, || Value::String("replaced".into()));
        assert_eq!(
            result,
            json!({"bar": {"baz": 1}, "foo": {"bar": "replaced", "qux": "replaced"}})
        );
    }
}
