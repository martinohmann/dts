mod de;
mod from;
mod ser;

use crate::value::Value;
use std::slice::Iter;
use std::vec::IntoIter;

/// The body of a HCL config file or block.
#[derive(Debug, PartialEq, Clone)]
pub struct Body {
    inner: Vec<Structure>,
}

impl Body {
    pub fn iter(&self) -> StructureIter {
        StructureIter {
            inner: self.inner.iter(),
        }
    }

    pub fn attributes(&self) -> BlockIter {
        BlockIter {
            inner: self.inner.iter(),
        }
    }

    pub fn blocks(&self) -> BlockIter {
        BlockIter {
            inner: self.inner.iter(),
        }
    }

    pub fn has_attributes(&self) -> bool {
        self.iter().any(|s| s.is_attribute())
    }

    pub fn has_blocks(&self) -> bool {
        self.iter().any(|s| s.is_block())
    }
}

impl FromIterator<Structure> for Body {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Structure>,
    {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Body {
    type Item = Structure;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

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

    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(block) => Some(block),
            Self::Attribute(_) => None,
        }
    }

    pub fn is_attribute(&self) -> bool {
        matches!(self, Self::Attribute(_))
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
    pub fn new(key: &str, value: Value) -> Self {
        Self {
            key: key.to_owned(),
            value,
        }
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
    ident: String,
    keys: Vec<String>,
    body: Body,
}

impl Block {
    pub fn new<K, B>(ident: &str, keys: K, body: B) -> Self
    where
        K: IntoIterator<Item = String>,
        B: IntoIterator<Item = Structure>,
    {
        Self {
            ident: ident.to_owned(),
            keys: keys.into_iter().collect(),
            body: body.into_iter().collect(),
        }
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }

    pub fn keys(&self) -> &Vec<String> {
        &self.keys
    }

    pub fn body(&self) -> &Body {
        &self.body
    }
}

pub struct StructureIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for StructureIter<'a> {
    type Item = &'a Structure;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct AttributeIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for AttributeIter<'a> {
    type Item = &'a Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            match structure.as_attribute() {
                Some(attr) => return Some(attr),
                None => continue,
            }
        }

        None
    }
}

pub struct BlockIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            match structure.as_block() {
                Some(block) => return Some(block),
                None => continue,
            }
        }

        None
    }
}
