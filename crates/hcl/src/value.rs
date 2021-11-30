/// The map type used for objects.
pub type Map<K, V> = std::collections::HashMap<K, V>;

/// The body of a HCL config file or block.
pub type Body = Vec<Structure>;

/// Represents a HCL Structures.
#[derive(Debug, PartialEq)]
pub enum Structure {
    /// An Attribute is a key-value pair where the key is a string identifier. The value can be a
    /// literal value or complex expression.
    Attribute(String, Value),
    /// A nested block which has an identifier, zero or more keys and a body.
    Block(Block),
}

/// A nested block which has an identifier, zero or more keys and a body.
#[derive(Debug, PartialEq)]
pub struct Block {
    ident: String,
    keys: Vec<String>,
    body: Body,
}

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

/// Represents a HCL number.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Represents a integer.
    Int(i64),
    /// Represents a float.
    Float(f64),
}
