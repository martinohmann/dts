use crate::number::Number;

/// The map type used for objects.
pub type Map<K, V> = std::collections::HashMap<K, V>;

/// Represents any valid HCL value.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// Represents a HCL null value.
    Null,
    /// Represents a HCL boolean.
    Bool(bool),
    /// Represents a HCL number, either integer or float.
    Number(Number),
    /// Represents a HCL string.
    String(String),
    /// Represents a HCL list.
    List(Vec<Value>),
    /// Represents a HCL object.
    Object(Map<String, Value>),
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}
