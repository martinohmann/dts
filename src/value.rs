use serde::ser::{Serialize, Serializer};
use std::collections::HashMap;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Yaml(serde_yaml::Value),
    MultiYaml(Vec<serde_yaml::Value>),
    Json(serde_json::Value),
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
