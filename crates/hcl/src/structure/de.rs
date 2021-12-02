use super::Structure;
use crate::value::{Map, Value};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for Structure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Structure;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL structure")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Map::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }

                Value::Object(map).try_into().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::de::from_str;
    use crate::structure::{Attribute, Block, Body};
    use crate::value::Value;

    #[test]
    fn deserialize_structure() {
        let hcl = r#"
            foo = 42

            block {
              bar = true
              baz = [var.enabled, 1, "two"]
            }
        "#;
        let body: Body = from_str(hcl).unwrap();
        let expected = vec![
            Attribute::new("foo".into(), 42.into()).into(),
            Block::new(
                vec!["block".into()],
                vec![
                    Attribute::new("bar".into(), true.into()).into(),
                    Attribute::new(
                        "baz".into(),
                        Value::Array(vec!["${var.enabled}".into(), 1.into(), "two".into()]),
                    )
                    .into(),
                ],
            )
            .into(),
        ];

        assert_eq!(body, expected);
    }
}
