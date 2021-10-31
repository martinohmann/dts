use anyhow::{bail, Result};
use serde::ser::{Serialize, Serializer};
use serde_json as json;
use serde_yaml as yaml;
use std::collections::HashMap;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Yaml(yaml::Value),
    MultiYaml(Vec<yaml::Value>),
    Json(json::Value),
    Ron(ron::Value),
    Toml(toml::Value),
    Csv(Vec<Vec<String>>),
    CsvHeaders(Vec<HashMap<String, String>>),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Yaml(ref v) => v.serialize(serializer),
            Value::MultiYaml(ref v) => v.serialize(serializer),
            Value::Json(ref v) => v.serialize(serializer),
            Value::Ron(ref v) => v.serialize(serializer),
            Value::Toml(ref v) => v.serialize(serializer),
            Value::Csv(ref v) => v.serialize(serializer),
            Value::CsvHeaders(ref v) => v.serialize(serializer),
        }
    }
}

impl Value {
    pub fn is_array(&self) -> bool {
        match self {
            Value::Yaml(yaml::Value::Sequence(_)) => true,
            Value::MultiYaml(_) => true,
            Value::Json(json::Value::Array(_)) => true,
            Value::Ron(ron::Value::Seq(_)) => true,
            Value::Toml(toml::Value::Array(_)) => true,
            Value::Csv(_) => true,
            Value::CsvHeaders(_) => true,
            _ => false,
        }
    }

    pub fn to_vec(&self) -> Result<Vec<json::Value>> {
        match self {
            Value::Json(json::Value::Array(v)) => Ok(v.clone()),
            Value::MultiYaml(ref v) => to_vec(v),
            Value::Yaml(yaml::Value::Sequence(ref v)) => to_vec(v),
            Value::Ron(ron::Value::Seq(ref v)) => to_vec(v),
            Value::Toml(toml::Value::Array(ref v)) => to_vec(v),
            Value::Csv(ref v) => to_vec(v),
            Value::CsvHeaders(ref v) => to_vec(v),
            _ => bail!("value isn't structured like an array"),
        }
    }
}

fn to_vec<T>(value: T) -> Result<Vec<json::Value>>
where
    T: IntoIterator,
    T::Item: Serialize,
{
    value.into_iter().map(|v| Ok(json::to_value(v)?)).collect()
}
