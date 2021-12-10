use super::TransformError;
use crate::Value;
use serde_json::{Map, Number};
use std::cmp::{self, Ordering};
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
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            other => Err(TransformError::invalid_sort_order(other)),
        }
    }
}

/// A type that can sort `Value` instances.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueSorter {
    order: Order,
    max_depth: Option<u64>,
}

impl ValueSorter {
    /// Creates a new `ValueSorter` which sorts values passed to the `sort` method using the given
    /// `Order` and `max_depth`.
    ///
    /// If `max_depth` is `Some`, child collections are sorted until the given depth. If
    /// `max_depth` is `None`, the `ValueSorter` will recursively visit all child collections and
    /// sort them. A `max_depth` of 0 only sorts the first level.
    pub fn new(order: Order, max_depth: Option<u64>) -> ValueSorter {
        ValueSorter { order, max_depth }
    }

    /// Sorts value if it is of variant `Value::Array` or `Value::Object`. For any other `Value`
    /// variant this is a no-op.
    pub fn sort(&self, value: &mut Value) {
        self.sort_impl(value, 0)
    }

    fn sort_impl(&self, value: &mut Value, depth: u64) {
        match value {
            Value::Array(array) => self.sort_array(array, depth),
            Value::Object(object) => self.sort_object(object, depth),
            _ => (),
        }
    }

    fn sort_array(&self, array: &mut Vec<Value>, depth: u64) {
        if self.recurse(depth) {
            array.iter_mut().for_each(|v| self.sort_impl(v, depth + 1));
        }

        array.sort_by(|lhs, rhs| self.cmp_values(lhs, rhs));
    }

    fn sort_object(&self, object: &mut Map<String, Value>, depth: u64) {
        if self.recurse(depth) {
            object
                .values_mut()
                .for_each(|v| self.sort_impl(v, depth + 1));
        }

        let mut sortable: Vec<(&String, &Value)> = object.iter().collect();

        sortable.sort_by(|lhs, rhs| match self.cmp_strings(lhs.0, rhs.0) {
            Ordering::Equal => self.cmp_values(lhs.1, rhs.1),
            non_eq => non_eq,
        });

        *object = sortable
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }

    fn recurse(&self, depth: u64) -> bool {
        self.max_depth
            .map(|max_depth| depth < max_depth)
            .unwrap_or(true)
    }

    fn cmp_values(&self, lhs: &Value, rhs: &Value) -> Ordering {
        match self.order {
            Order::Asc => cmp_values(lhs, rhs),
            Order::Desc => cmp_values(rhs, lhs),
        }
    }

    fn cmp_strings(&self, lhs: &str, rhs: &str) -> Ordering {
        match self.order {
            Order::Asc => lhs.cmp(rhs),
            Order::Desc => rhs.cmp(lhs),
        }
    }
}

impl Default for ValueSorter {
    fn default() -> Self {
        ValueSorter::new(Order::Asc, None)
    }
}

impl FromStr for ValueSorter {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (order, max_depth) = match s.split_once(':') {
            Some((order, max_depth)) => (Order::from_str(order)?, Some(max_depth.parse()?)),
            None => (Order::from_str(s)?, None),
        };

        Ok(ValueSorter::new(order, max_depth))
    }
}

// Compares two values.
//
// `Value` is neither `PartialOrd` nor `Ord`, so we use a custom comparator.
fn cmp_values(lhs: &Value, rhs: &Value) -> Ordering {
    match lhs {
        Value::Null => match rhs {
            Value::Null => Ordering::Equal,
            _ => Ordering::Less,
        },
        Value::Bool(x) => match rhs {
            Value::Bool(y) => x.cmp(y),
            Value::Null => Ordering::Greater,
            _ => Ordering::Less,
        },
        Value::Number(x) => match rhs {
            Value::Number(y) => cmp_numbers(x, y),
            Value::Null | Value::Bool(_) => Ordering::Greater,
            _ => Ordering::Less,
        },
        Value::String(x) => match rhs {
            Value::String(y) => x.cmp(y),
            Value::Array(_) | Value::Object(_) => Ordering::Less,
            _ => Ordering::Greater,
        },
        Value::Object(x) => match rhs {
            Value::Object(y) => cmp_maps(x, y),
            Value::Array(_) => Ordering::Less,
            _ => Ordering::Greater,
        },
        Value::Array(x) => match rhs {
            Value::Array(y) => cmp_arrays(x, y),
            _ => Ordering::Greater,
        },
    }
}

// Compares two maps.
//
// This assumes that the underlying `Map` implementation has a predictable order like
// `std::collections::BTreeMap` or `indexmap::IndexMap`.
fn cmp_maps(lhs: &Map<String, Value>, rhs: &Map<String, Value>) -> Ordering {
    match lhs.len().cmp(&rhs.len()) {
        Ordering::Equal => {
            for (lhs, rhs) in lhs.iter().zip(rhs.iter()) {
                match cmp_kv_pairs(&lhs, &rhs) {
                    Ordering::Equal => (),
                    non_eq => return non_eq,
                }
            }

            Ordering::Equal
        }
        non_eq => non_eq,
    }
}

// Compares two key-value pairs.
fn cmp_kv_pairs(lhs: &(&String, &Value), rhs: &(&String, &Value)) -> Ordering {
    match lhs.0.cmp(rhs.0) {
        Ordering::Equal => cmp_values(lhs.1, rhs.1),
        non_eq => non_eq,
    }
}

// Compares two arrays.
fn cmp_arrays(lhs: &[Value], rhs: &[Value]) -> Ordering {
    let l = cmp::min(lhs.len(), rhs.len());

    let left = &lhs[..l];
    let right = &rhs[..l];

    for i in 0..l {
        match cmp_values(&left[i], &right[i]) {
            Ordering::Equal => (),
            non_eq => return non_eq,
        }
    }

    lhs.len().cmp(&rhs.len())
}

// Compares two numbers.
fn cmp_numbers(lhs: &Number, rhs: &Number) -> Ordering {
    lhs.as_f64()
        .and_then(|x| rhs.as_f64().and_then(|y| x.partial_cmp(&y)))
        .unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::{json, to_string_pretty};

    #[track_caller]
    fn assert_eq_sorted(order: Order, max_depth: Option<u64>, mut given: Value, expected: Value) {
        let sorter = ValueSorter::new(order, max_depth);
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

        assert_eq_sorted(Order::Asc, Some(0), value, expected);
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

        assert_eq_sorted(Order::Desc, Some(0), value, expected);
    }

    #[test]
    fn recurse() {
        let value = json!([
            {"foo": [{"qux": [2, 1, 7], "bar": [{"baz": [4, 1]}, {"bar": [3, 1]}]}]},
            {"foo": [{"qux": [4, 7, 1]}]}
        ]);

        let expected = json!([
            {"foo": [{"qux": [4, 7, 1]}]},
            {"foo": [{"qux": [2, 1, 7], "bar": [{"baz": [4, 1]}, {"bar": [3, 1]}]}]}
        ]);

        assert_eq_sorted(Order::Asc, Some(0), value.clone(), expected);

        let expected = json!([
            {"foo": [{"qux": [1, 4, 7]}]},
            {"foo": [{"bar": [{"bar": [3, 1]}, {"baz": [4, 1]}], "qux": [1, 2, 7]}]}
        ]);

        assert_eq_sorted(Order::Asc, Some(5), value.clone(), expected);

        let expected = json!([
            {"foo": [{"qux": [1, 4, 7]}]},
            {"foo": [{"bar": [{"bar": [1, 3]}, {"baz": [1, 4]}], "qux": [1, 2, 7]}]}
        ]);

        assert_eq_sorted(Order::Asc, None, value.clone(), expected);
    }
}
