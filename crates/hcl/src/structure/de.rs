use super::Body;
use crate::value::Value;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for Body {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Body;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL config file or block")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Value::Array(vec).try_into().map_err(de::Error::custom)
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
        let expected = Body::from_iter(vec![
            Attribute::new("foo".into(), 42.into()).into(),
            Block::new(
                "block",
                vec![],
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
        ]);

        assert_eq!(body, expected);
    }
}
