use super::{Attribute, Block, Structure};
use crate::value::Value;
use serde::ser::{Serialize, Serializer};

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Attribute(attr) => attr.serialize(serializer),
            Self::Block(block) => block.serialize(serializer),
        }
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Value::from(self).serialize(serializer)
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Value::from(self).serialize(serializer)
    }
}
