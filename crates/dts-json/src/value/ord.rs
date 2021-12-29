use super::{Map, Value};
use std::cmp::Ordering;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Value::Null => match other {
                Value::Null => Ordering::Equal,
                _ => Ordering::Less,
            },
            Value::Bool(x) => match other {
                Value::Bool(y) => x.cmp(y),
                Value::Null => Ordering::Greater,
                _ => Ordering::Less,
            },
            Value::Number(x) => match other {
                Value::Number(y) => x.cmp(y),
                Value::Null | Value::Bool(_) => Ordering::Greater,
                _ => Ordering::Less,
            },
            Value::String(x) => match other {
                Value::String(y) => x.cmp(y),
                Value::Array(_) | Value::Object(_) => Ordering::Less,
                _ => Ordering::Greater,
            },
            Value::Object(x) => match other {
                Value::Object(y) => cmp_maps(x, y),
                Value::Array(_) => Ordering::Less,
                _ => Ordering::Greater,
            },
            Value::Array(x) => match other {
                Value::Array(y) => cmp_arrays(x, y),
                _ => Ordering::Greater,
            },
        }
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
                match lhs.cmp(&rhs) {
                    Ordering::Equal => (),
                    non_eq => return non_eq,
                }
            }

            Ordering::Equal
        }
        non_eq => non_eq,
    }
}

// Compares two arrays.
fn cmp_arrays(lhs: &[Value], rhs: &[Value]) -> Ordering {
    match lhs.len().cmp(&rhs.len()) {
        Ordering::Equal => {
            for i in 0..lhs.len() {
                match lhs[i].cmp(&rhs[i]) {
                    Ordering::Equal => (),
                    non_eq => return non_eq,
                }
            }

            Ordering::Equal
        }
        non_eq => non_eq,
    }
}
