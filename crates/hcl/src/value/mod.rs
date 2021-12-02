mod de;
mod from;
mod ser;

use crate::number::Number;

/// The map type used for objects.
pub type Map<K, V> = std::collections::HashMap<K, V>;

/// Represents any valid HCL value.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Represents a HCL null value.
    Null,
    /// Represents a HCL boolean.
    Bool(bool),
    /// Represents a HCL number, either integer or float.
    Number(Number),
    /// Represents a HCL string.
    String(String),
    /// Represents a HCL array.
    Array(Vec<Value>),
    /// Represents a HCL object.
    Object(Map<String, Value>),
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

impl Value {
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_f64(&self) -> bool {
        self.as_number().map(Number::is_f64).unwrap_or(false)
    }

    pub fn is_i64(&self) -> bool {
        self.as_number().map(Number::is_i64).unwrap_or(false)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_null(&self) -> bool {
        *self == Value::Null
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_u64(&self) -> bool {
        self.as_number().map(Number::is_u64).unwrap_or(false)
    }

    pub fn take(&mut self) -> Value {
        std::mem::replace(self, Value::Null)
    }
}
