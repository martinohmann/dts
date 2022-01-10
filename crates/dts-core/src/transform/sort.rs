//! Provides a `ValueSorter` for recursively sorting `Value` instances.

use crate::Error;
use dts_json::Value;
use std::cmp::Ordering;
use std::str::FromStr;

/// Possible sort orders for the `ValueSorter`.
#[derive(Debug, PartialEq, Clone)]
pub enum Order {
    /// Sort values in ascending order.
    Asc,
    /// Sort values in descending order.
    Desc,
}

impl FromStr for Order {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            other => Err(Error::new(format!("Invalid sort order `{}`", other))),
        }
    }
}

/// A type that can sort `Value` instances.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueSorter {
    order: Order,
}

impl ValueSorter {
    /// Creates a new `ValueSorter` which sorts values passed to the `sort` method using the given
    /// `Order`.
    pub fn new(order: Order) -> ValueSorter {
        ValueSorter { order }
    }

    /// Sorts value if it is of variant `Value::Array` or `Value::Object`. For any other `Value`
    /// variant this is a no-op.
    pub fn sort(&self, value: &mut Value) {
        match value {
            Value::Array(array) => array.sort_by(|lhs, rhs| self.compare(lhs, rhs)),
            Value::Object(object) => {
                object.sort_by(|k1, v1, k2, v2| self.compare(&(k1, v1), &(k2, v2)))
            }
            _ => (),
        }
    }

    fn compare<T: Ord>(&self, lhs: &T, rhs: &T) -> Ordering {
        match self.order {
            Order::Asc => lhs.cmp(rhs),
            Order::Desc => rhs.cmp(lhs),
        }
    }
}

impl Default for ValueSorter {
    fn default() -> Self {
        ValueSorter::new(Order::Asc)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;
    use pretty_assertions::assert_eq;
    use serde_json::to_string_pretty;

    #[track_caller]
    fn assert_eq_sorted(order: Order, mut given: Value, expected: Value) {
        let sorter = ValueSorter::new(order);
        sorter.sort(&mut given);

        assert_eq!(
            to_string_pretty(&given).unwrap(),
            to_string_pretty(&expected).unwrap()
        );
    }

    #[test]
    fn sort_asc() {
        let value = json!([
            {"bar": "baz", "qux": 1},
            {"bar": "baz"},
            {"foo": "baz", "bar": [42, 3, 13]},
            {"foo": "bar", "bar": ["one", "two", "three"]}
        ]);

        let expected = json!([
            {"bar": "baz"},
            {"bar": "baz", "qux": 1},
            {"foo": "bar", "bar": ["one", "two", "three"]},
            {"foo": "baz", "bar": [42, 3, 13]},
        ]);

        assert_eq_sorted(Order::Asc, value, expected);
    }

    #[test]
    fn sort_desc() {
        let value = json!([
            {"bar": "baz", "qux": 1},
            {"bar": "baz"},
            {"foo": "baz", "bar": [42, 3, 13]},
            {"foo": "bar", "bar": ["one", "two", "three"]}
        ]);

        let expected = json!([
            {"foo": "baz", "bar": [42, 3, 13]},
            {"foo": "bar", "bar": ["one", "two", "three"]},
            {"bar": "baz", "qux": 1},
            {"bar": "baz"},
        ]);

        assert_eq_sorted(Order::Desc, value, expected);
    }
}
