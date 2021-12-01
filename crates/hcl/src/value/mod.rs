mod de;
mod from;
mod ser;

use crate::number::Number;
use crate::structure::Structure;
use crate::Result;

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
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn into_structure(self) -> Result<Structure> {
        self.try_into()
    }
}
