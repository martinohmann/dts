use super::{Attribute, Block, Body, Structure};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for structure in self.iter() {
            seq.serialize_element(structure)?;
        }

        seq.end()
    }
}

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
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("kind", "attribute")?;
        map.serialize_entry("key", self.key())?;
        map.serialize_entry("value", self.value())?;
        map.end()
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("kind", "block")?;
        map.serialize_entry("ident", self.ident())?;
        map.serialize_entry("keys", self.keys())?;
        map.serialize_entry("body", self.body())?;
        map.end()
    }
}
