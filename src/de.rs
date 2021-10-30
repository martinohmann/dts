use crate::Encoding;
use anyhow::Result;
use erased_serde::Serialize;
use serde::Deserialize;

#[derive(Debug)]
pub struct DeserializeOptions {
    pub encoding: Encoding,
    pub all_documents: bool,
}

pub struct Deserializer {
    opts: DeserializeOptions,
}

impl Deserializer {
    pub fn new(opts: DeserializeOptions) -> Self {
        Self { opts }
    }

    pub fn deserialize<R>(&self, reader: R) -> Result<Box<dyn Serialize>>
    where
        R: std::io::Read,
    {
        let mut reader = reader;

        match &self.opts.encoding {
            Encoding::Yaml => {
                let mut values = Vec::new();

                for doc in serde_yaml::Deserializer::from_reader(reader) {
                    let value = serde_yaml::Value::deserialize(doc)?;

                    if self.opts.all_documents {
                        values.push(value);
                    } else {
                        return Ok(Box::new(value));
                    }
                }

                Ok(Box::new(values))
            }
            Encoding::Json => {
                let value: serde_json::Value = serde_json::from_reader(reader)?;
                Ok(Box::new(value))
            }
            Encoding::Ron => {
                let value: ron::Value = ron::de::from_reader(reader)?;
                Ok(Box::new(value))
            }
            Encoding::Toml => {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                let value: toml::Value = toml::de::from_slice(&buf)?;
                Ok(Box::new(value))
            }
            Encoding::Json5 => {
                let mut s = String::new();
                reader.read_to_string(&mut s)?;
                let value: serde_json::Value = json5::from_str(&s)?;
                Ok(Box::new(value))
            }
            Encoding::Hjson => {
                let mut s = String::new();
                reader.read_to_string(&mut s)?;

                let value: serde_json::Value = deser_hjson::from_str(&s)?;
                Ok(Box::new(value))
            }
        }
    }
}
