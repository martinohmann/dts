use super::{Structure, Value};
use crate::value::Map;
use crate::Error;

impl TryFrom<Value> for Structure {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Object(object) => object.try_into(),
            Value::Array(array) => array.try_into(),
            _ => Err(Error::new("not a HCL structure")),
        }
    }
}

impl TryFrom<&Value> for Structure {
    type Error = Error;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        TryFrom::try_from(v.clone())
    }
}

impl TryFrom<Map<String, Value>> for Structure {
    type Error = Error;

    fn try_from(m: Map<String, Value>) -> Result<Self, Self::Error> {
        match m.len() {
            0 => Err(Error::new("attribute expected")),
            1 => {
                let (k, v) = m.into_iter().take(1).next().unwrap();
                Ok(Self::Attribute(k, v))
            }
            _ => Err(Error::new("ambiguous attribute")),
        }
    }
}

impl TryFrom<&Map<String, Value>> for Structure {
    type Error = Error;

    fn try_from(m: &Map<String, Value>) -> Result<Self, Self::Error> {
        TryFrom::try_from(m.clone())
    }
}

impl TryFrom<Vec<Value>> for Structure {
    type Error = Error;

    fn try_from(v: Vec<Value>) -> Result<Self, Self::Error> {
        match v.len() {
            2 => {
                let ident = match &v[0] {
                    Value::Array(array) => {
                        if array.iter().all(Value::is_string) {
                            array.iter().filter_map(Value::as_string).cloned().collect()
                        } else {
                            return Err(Error::new("block identifiers must be strings"));
                        }
                    }
                    _ => return Err(Error::new("not a block identifier")),
                };

                let body = match &v[1] {
                    Value::Array(array) => array
                        .iter()
                        .map(TryFrom::try_from)
                        .collect::<Result<_, _>>()?,
                    _ => return Err(Error::new("not a block body")),
                };

                Ok(Self::Block(ident, body))
            }
            _ => Err(Error::new("not a block")),
        }
    }
}

impl TryFrom<&Vec<Value>> for Structure {
    type Error = Error;

    fn try_from(v: &Vec<Value>) -> Result<Self, Self::Error> {
        TryFrom::try_from(v.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn attribute_from_value() {
        let value = Value::Object(hashmap! {
            "foo".into() => Value::String("bar".into())
        });

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Attribute("foo".into(), Value::String("bar".into()))
        );

        let value = Value::Object(Map::new());

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "foo".into() => Value::String("bar".into()),
            "bar".into() => Value::Bool(true)
        });

        assert!(Structure::try_from(value).is_err());
    }

    #[test]
    fn block_from_value() {
        let value = Value::Array(vec![
            Value::Array(vec![
                Value::String("resource".into()),
                Value::String("aws_s3_bucket".into()),
                Value::String("mybucket".into()),
            ]),
            Value::Array(vec![Value::Object(hashmap! {
                "name".into() => Value::String("mybucket".into())
            })]),
        ]);

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Block(
                vec!["resource".into(), "aws_s3_bucket".into(), "mybucket".into()],
                vec![Structure::Attribute(
                    "name".into(),
                    Value::String("mybucket".into())
                )]
            )
        );

        let value = Value::Array(vec![
            Value::Array(vec![Value::String("resource".into()), Value::Bool(true)]),
            Value::Array(vec![Value::Object(hashmap! {
                "name".into() => Value::String("mybucket".into())
            })]),
        ]);

        assert!(Structure::try_from(value).is_err());

        let value = Value::Array(vec![
            Value::String("resource".into()),
            Value::Array(vec![Value::Object(hashmap! {
                "name".into() => Value::String("mybucket".into())
            })]),
        ]);

        assert!(Structure::try_from(value).is_err());

        let value = Value::Array(vec![
            Value::Array(vec![
                Value::String("resource".into()),
                Value::String("aws_s3_bucket".into()),
                Value::String("mybucket".into()),
            ]),
            Value::Array(vec![Value::Null]),
        ]);

        assert!(Structure::try_from(value).is_err());

        let value = Value::Array(Vec::new());

        assert!(Structure::try_from(value).is_err());
    }
}
