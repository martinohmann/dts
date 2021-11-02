use crate::{Encoding, Result, Value};
use serde::Deserialize;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeserializeOptions {
    pub all_documents: bool,
    pub csv_without_headers: bool,
    pub csv_headers_as_keys: bool,
}

impl DeserializeOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct DeserializerBuilder {
    opts: DeserializeOptions,
}

impl DeserializerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all_documents(&mut self, all_documents: bool) -> &mut Self {
        self.opts.all_documents = all_documents;
        self
    }

    pub fn csv_without_headers(&mut self, yes: bool) -> &mut Self {
        self.opts.csv_without_headers = yes;
        self
    }

    pub fn csv_headers_as_keys(&mut self, yes: bool) -> &mut Self {
        self.opts.csv_headers_as_keys = yes;
        self
    }

    pub fn build(&self, encoding: Encoding) -> Deserializer {
        Deserializer::new(encoding, self.opts.clone())
    }
}

pub struct Deserializer {
    encoding: Encoding,
    opts: DeserializeOptions,
}

impl Deserializer {
    pub fn new(encoding: Encoding, opts: DeserializeOptions) -> Self {
        Self { encoding, opts }
    }

    pub fn deserialize<R>(&self, reader: &mut R) -> Result<Value>
    where
        R: std::io::Read,
    {
        match &self.encoding {
            Encoding::Yaml => deserialize_yaml(reader, &self.opts),
            Encoding::Json => deserialize_json(reader),
            Encoding::Ron => deserialize_ron(reader),
            Encoding::Toml => deserialize_toml(reader),
            Encoding::Json5 => deserialize_json5(reader),
            Encoding::Hjson => deserialize_hjson(reader),
            Encoding::Csv => deserialize_csv(reader, b',', &self.opts),
            Encoding::Tsv => deserialize_csv(reader, b'\t', &self.opts),
            Encoding::Pickle => deserialize_pickle(reader),
            Encoding::QueryString => deserialize_query_string(reader),
        }
    }
}

fn deserialize_yaml<R>(reader: &mut R, opts: &DeserializeOptions) -> Result<Value>
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

fn deserialize_json<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(serde_json::from_reader(reader)?)
}

fn deserialize_ron<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(ron::de::from_reader(reader)?)
}

fn deserialize_toml<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(toml::de::from_slice(&buf)?)
}

fn deserialize_json5<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(json5::from_str(&s)?)
}

fn deserialize_hjson<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(deser_hjson::from_str(&s)?)
}

fn deserialize_csv<R>(reader: &mut R, delimiter: u8, opts: &DeserializeOptions) -> Result<Value>
where
    R: std::io::Read,
{
    let keep_first_line = opts.csv_without_headers || opts.csv_headers_as_keys;

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(!keep_first_line)
        .delimiter(delimiter)
        .from_reader(reader);

    let mut iter = csv_reader.deserialize();

    let value = if opts.csv_headers_as_keys {
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
                    .collect::<Result<_>>()?,
                )
            }
            None => Value::Array(Vec::new()),
        }
    } else {
        Value::Array(
            iter.map(|v| Ok(serde_json::to_value(v?)?))
                .collect::<Result<Vec<Value>>>()?,
        )
    };

    Ok(value)
}

fn deserialize_pickle<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(serde_pickle::from_reader(reader, Default::default())?)
}

fn deserialize_query_string<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(Value::Object(serde_qs::from_str(&s)?))
}
