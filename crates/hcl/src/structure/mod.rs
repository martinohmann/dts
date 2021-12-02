mod de;
mod from;
mod ser;

use crate::value::Value;

/// The body of a HCL config file or block.
pub type Body = Vec<Structure>;

/// Represents a HCL structure.
#[derive(Debug, PartialEq, Clone)]
pub enum Structure {
    /// An Attribute is a key-value pair where the key is a string identifier. The value can be a
    /// literal value or complex expression.
    Attribute(Attribute),
    /// A nested block which has an identifier, zero or more keys and a body.
    Block(Block),
}

impl Structure {
    pub fn as_attribute(&self) -> Option<&Attribute> {
        match self {
            Self::Attribute(attr) => Some(attr),
            Self::Block(_) => None,
        }
    }

    pub fn is_attribute(&self) -> bool {
        matches!(self, Self::Attribute(_))
    }

    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(block) => Some(block),
            Self::Attribute(_) => None,
        }
    }

    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

/// Represents a HCL attribute.
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    key: String,
    value: Value,
}

impl Attribute {
    pub fn new(key: String, value: Value) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

/// Represents a HCL block.
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    ident: Vec<String>,
    body: Body,
}

impl Block {
    pub fn new(ident: Vec<String>, body: Body) -> Self {
        Self { ident, body }
    }

    pub fn ident(&self) -> &[String] {
        &self.ident
    }

    pub fn body(&self) -> &Body {
        &self.body
    }
}
