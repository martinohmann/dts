use crate::value::Value;

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
