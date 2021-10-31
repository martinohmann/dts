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
        match &self.encoding {
            Encoding::Yaml => deserialize_yaml(reader, opts),
            Encoding::Json => deserialize_json(reader),
            Encoding::Ron => deserialize_ron(reader),
            Encoding::Toml => deserialize_toml(reader),
            Encoding::Json5 => deserialize_json5(reader),
            Encoding::Hjson => deserialize_hjson(reader),
            Encoding::Csv => deserialize_csv(reader, b',', opts),
            Encoding::Tsv => deserialize_csv(reader, b'\t', opts),
        }
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

fn deserialize_json<R>(reader: R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(serde_json::from_reader(reader)?)
}

fn deserialize_ron<R>(reader: R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(ron::de::from_reader(reader)?)
}

fn deserialize_toml<R>(reader: R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut reader = reader;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(toml::de::from_slice(&buf)?)
}

fn deserialize_json5<R>(reader: R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut reader = reader;
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(json5::from_str(&s)?)
}

fn deserialize_hjson<R>(reader: R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut reader = reader;
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(deser_hjson::from_str(&s)?)
}

fn deserialize_csv<R>(reader: R, delimiter: u8, opts: DeserializeOptions) -> Result<Value>
where
    R: std::io::Read,
{
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(reader);

    let mut iter = csv_reader.deserialize();

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
