//! Provides types to visit elements of collections.

use super::Transform;
use dts_json::Value;

/// A trait for visiting array values and key-value pairs of objects.
pub trait Visitor {
    /// Visits a key of an object and produces a new `String`.
    ///
    /// The default implementation just returns the key unchanged.
    fn visit_key(&self, key: String) -> String {
        key
    }

    /// Visits an array or object value and produces a new `Value`.
    ///
    /// The default implementation just returns the value unchanged.
    fn visit_value(&self, value: Value) -> Value {
        value
    }
}

/// A `Visitor` that applies a transformation to object keys only.
pub struct KeyVisitor<T>(T);

impl<T> KeyVisitor<T> {
    /// Creates a new `KeyVisitor`.
    pub fn new(expr: T) -> Self {
        KeyVisitor(expr)
    }
}

impl<T> Visitor for KeyVisitor<T>
where
    T: Transform,
{
    fn visit_key(&self, key: String) -> String {
        self.0.transform(key.into()).into_string()
    }
}

/// A `Visitor` that applies a transformation to array and object values.
pub struct ValueVisitor<T>(T);

impl<T> ValueVisitor<T> {
    /// Creates a new `ValueVisitor`.
    pub fn new(expr: T) -> Self {
        ValueVisitor(expr)
    }
}

impl<T> Visitor for ValueVisitor<T>
where
    T: Transform,
{
    fn visit_value(&self, value: Value) -> Value {
        self.0.transform(value)
    }
}
