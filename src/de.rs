use crate::Encoding;
use anyhow::Result;
use erased_serde::Serialize;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeserializeOptions {
    pub all_documents: bool,
    pub no_headers: bool,
}

pub struct Deserializer {
    encoding: Encoding,
}

impl Deserializer {
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }

    pub fn deserialize<R>(&self, reader: R, opts: DeserializeOptions) -> Result<Box<dyn Serialize>>
    where
        R: std::io::Read,
    {
        let mut reader = reader;

        match &self.encoding {
            Encoding::Yaml => {
                let mut values = Vec::new();

                for doc in serde_yaml::Deserializer::from_reader(reader) {
                    let value = serde_yaml::Value::deserialize(doc)?;

                    if opts.all_documents {
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
            Encoding::Csv => {
                let mut csv_reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(reader);
                deserialize_csv(&mut csv_reader, opts)
            }
            Encoding::Tsv => {
                let mut tsv_reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .delimiter(b'\t')
                    .from_reader(reader);
                deserialize_csv(&mut tsv_reader, opts)
            }
        }
    }
}

fn deserialize_csv<R>(
    reader: &mut csv::Reader<R>,
    opts: DeserializeOptions,
) -> Result<Box<dyn Serialize>>
where
    R: std::io::Read,
{
    let mut iter = reader.deserialize();

    if opts.no_headers {
        let value: Vec<Vec<String>> = iter.collect::<Result<_, _>>()?;

        return Ok(Box::new(value));
    }

    match iter.next() {
        Some(headers) => {
            let headers: Vec<String> = headers?;

            let value: Vec<BTreeMap<String, String>> = iter
                .map(|record| {
                    Ok(headers
                        .iter()
                        .zip(record?.iter())
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect())
                })
                .collect::<Result<_, csv::Error>>()?;

            Ok(Box::new(value))
        }
        None => Ok(Box::new(Vec::<()>::new())),
    }
}
