use super::TransformError;
use crate::{Map, Value};
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

        array.sort_by(|lhs, rhs| self.compare(lhs, rhs));
    }

    fn sort_object(&self, object: &mut Map<String, Value>, depth: u64) {
        if self.recurse(depth) {
            object
                .values_mut()
                .for_each(|v| self.sort_impl(v, depth + 1));
        }

        let mut sortable: Vec<(&String, &Value)> = object.iter().collect();

        sortable.sort_by(|lhs, rhs| self.compare(lhs, rhs));

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

    fn compare<T: Ord>(&self, lhs: &T, rhs: &T) -> Ordering {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use pretty_assertions::assert_eq;
    use serde_json::to_string_pretty;

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
