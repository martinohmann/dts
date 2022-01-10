//! Provides types to visit elements of collections.

use super::{Chain, Transform};
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

/// A `Visitor` that applies a chain of transformations to object keys only.
pub struct KeyVisitor(Chain);

impl KeyVisitor {
    /// Creates a new `KeyVisitor`.
    pub fn new(chain: Chain) -> Self {
        KeyVisitor(chain)
    }
}

impl Visitor for KeyVisitor {
    fn visit_key(&self, key: String) -> String {
        self.0.transform(key.into()).into_string()
    }
}

/// A `Visitor` that applies a chain of transformations to array and object values.
pub struct ValueVisitor(Chain);

impl ValueVisitor {
    /// Creates a new `ValueVisitor`.
    pub fn new(chain: Chain) -> Self {
        ValueVisitor(chain)
    }
}

impl Visitor for ValueVisitor {
    fn visit_value(&self, value: Value) -> Value {
        self.0.transform(value)
    }
}
