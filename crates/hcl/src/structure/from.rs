use super::{Attribute, Block, Structure, Value};
use crate::Error;

impl TryFrom<&Value> for Structure {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Object(object) => match object.get("kind") {
                Some(Value::String(kind)) => match kind.as_str() {
                    "attribute" => Attribute::try_from(v).map(Structure::Attribute),
                    "block" => Block::try_from(v).map(Structure::Block),
                    kind => Err(Error::new(format!("invalid HCL structure kind `{}`", kind))),
                },
                _ => Err(Error::new("not a HCL structure")),
            },
            _ => Err(Error::new("object expected")),
        }
    }
}

impl TryFrom<Value> for Structure {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        TryFrom::try_from(&v)
    }
}

impl TryFrom<&Value> for Attribute {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Object(object) => {
                let key = match object.get("key") {
                    Some(Value::String(key)) => key.clone(),
                    _ => return Err(Error::new("not an attribute key")),
                };

                let value = match object.get("value") {
                    Some(value) => value.clone(),
                    _ => return Err(Error::new("not an attribute value")),
                };

                Ok(Attribute::new(key, value))
            }
            _ => Err(Error::new("not a HCL attribute")),
        }
    }
}

impl TryFrom<Value> for Attribute {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        TryFrom::try_from(&v)
    }
}

impl TryFrom<&Value> for Block {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Object(object) => {
                let ident = match object.get("ident") {
                    Some(Value::Array(array)) => {
                        if array.iter().all(Value::is_string) {
                            array.iter().filter_map(Value::as_string).cloned().collect()
                        } else {
                            return Err(Error::new("block identifiers must be strings"));
                        }
                    }
                    _ => return Err(Error::new("not a block identifier")),
                };

                let body = match object.get("body") {
                    Some(Value::Array(array)) => array
                        .iter()
                        .map(TryFrom::try_from)
                        .collect::<Result<_, _>>()?,
                    _ => return Err(Error::new("not a block body")),
                };

                Ok(Block::new(ident, body))
            }
            _ => Err(Error::new("not a HCL block")),
        }
    }
}

impl TryFrom<Value> for Block {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        TryFrom::try_from(&v)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::value::Map;
    use maplit::hashmap;

    #[test]
    fn attribute_from_value() {
        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "key".into() => "foo".into(),
            "value".into() => "bar".into()
        });

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Attribute(Attribute::new("foo".into(), Value::String("bar".into())))
        );

        let value = Value::Object(Map::new());

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "key".into() => "foo".into(),
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "value".into() => "bar".into()
        });

        assert!(Structure::try_from(value).is_err());
    }

    #[test]
    fn block_from_value() {
        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => Value::Array(vec![
                "resource".into(),
                "aws_s3_bucket".into(),
                "mybucket".into()
            ]),
            "body".into() => Value::Array(vec![
                Value::Object(hashmap! {
                    "kind".into() => "attribute".into(),
                    "key".into() => "name".into(),
                    "value".into() => "mybucket".into()
                })
            ])
        });

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Block(Block::new(
                vec!["resource".into(), "aws_s3_bucket".into(), "mybucket".into()],
                vec![Structure::Attribute(Attribute::new(
                    "name".into(),
                    Value::String("mybucket".into())
                ))]
            ))
        );

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "body".into() => Value::Array(vec![
                Value::Object(hashmap! {
                    "kind".into() => "attribute".into(),
                    "key".into() => "name".into(),
                    "value".into() => "mybucket".into()
                })
            ])
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => Value::Array(vec!["foo".into()]),
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => Value::Array(vec!["foo".into()]),
            "body".into() => Value::Array(vec![Value::Null])
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Array(Vec::new());

        assert!(Structure::try_from(value).is_err());
    }
}
