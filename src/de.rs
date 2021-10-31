use crate::{Encoding, Value};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeserializeOptions {
    pub all_documents: bool,
    pub headers: bool,
}

pub struct Deserializer {
    encoding: Encoding,
}

impl Deserializer {
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }

    pub fn deserialize<R>(&self, reader: R, opts: DeserializeOptions) -> Result<Value>
    where
        R: std::io::Read,
    {
        let mut reader = reader;

        let value = match &self.encoding {
            Encoding::Yaml => deserialize_yaml(reader, opts)?,
            Encoding::Json => serde_json::from_reader(reader)?,
            Encoding::Ron => ron::de::from_reader(reader)?,
            Encoding::Toml => {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                toml::de::from_slice(&buf)?
            }
            Encoding::Json5 => {
                let mut s = String::new();
                reader.read_to_string(&mut s)?;
                json5::from_str(&s)?
            }
            Encoding::Hjson => {
                let mut s = String::new();
                reader.read_to_string(&mut s)?;
                deser_hjson::from_str(&s)?
            }
            Encoding::Csv => {
                let mut csv_reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(reader);
                deserialize_csv(&mut csv_reader, opts)?
            }
            Encoding::Tsv => {
                let mut tsv_reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .delimiter(b'\t')
                    .from_reader(reader);
                deserialize_csv(&mut tsv_reader, opts)?
            }
        };

        Ok(value)
    }
}

fn deserialize_yaml<R>(reader: R, opts: DeserializeOptions) -> Result<Value>
where
    R: std::io::Read,
{
    let mut values = Vec::new();

    for doc in serde_yaml::Deserializer::from_reader(reader) {
        let value = Value::deserialize(doc)?;

        if opts.all_documents {
            values.push(value);
        } else {
            return Ok(value);
        }
    }

    Ok(Value::Array(values))
}

fn deserialize_csv<R>(reader: &mut csv::Reader<R>, opts: DeserializeOptions) -> Result<Value>
where
    R: std::io::Read,
{
    let mut iter = reader.deserialize();

    let value = if opts.headers {
        match iter.next() {
            Some(headers) => {
                let headers: Vec<String> = headers?;

                Value::Array(
                    iter.map(|record| {
                        Ok(headers
                            .iter()
                            .zip(record?.iter())
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect())
                    })
                    .collect::<Result<_, csv::Error>>()?,
                )
            }
            None => Value::Array(Vec::new()),
        }
    } else {
        Value::Array(
            iter.map(|v| Ok(serde_json::to_value(v?)?))
                .collect::<Result<Vec<Value>, anyhow::Error>>()?,
        )
    };

    Ok(value)
}
