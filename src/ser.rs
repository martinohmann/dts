use crate::{Encoding, Value};
use anyhow::{bail, Result};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SerializeOptions {
    pub pretty: bool,
    pub newline: bool,
}

impl SerializeOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct SerializerBuilder {
    encoding: Encoding,
    opts: SerializeOptions,
}

impl SerializerBuilder {
    pub fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            opts: SerializeOptions::new(),
        }
    }

    pub fn pretty(&mut self, pretty: bool) -> &mut Self {
        self.opts.pretty = pretty;
        self
    }

    pub fn newline(&mut self, newline: bool) -> &mut Self {
        self.opts.newline = newline;
        self
    }

    pub fn build(&self) -> Serializer {
        Serializer::new(self.encoding, self.opts.clone())
    }
}

pub struct Serializer {
    encoding: Encoding,
    opts: SerializeOptions,
}

impl Serializer {
    pub fn new(encoding: Encoding, opts: SerializeOptions) -> Self {
        Self { encoding, opts }
    }

    pub fn serialize<W>(&self, writer: &mut W, value: Value) -> Result<()>
    where
        W: std::io::Write,
    {
        match &self.encoding {
            Encoding::Yaml => serialize_yaml(writer, value)?,
            Encoding::Json | Encoding::Json5 => serialize_json(writer, value, &self.opts)?,
            Encoding::Ron => serialize_ron(writer, value, &self.opts)?,
            Encoding::Toml => serialize_toml(writer, value, &self.opts)?,
            Encoding::Csv => serialize_csv(writer, b',', value)?,
            Encoding::Tsv => serialize_csv(writer, b'\t', value)?,
            encoding => bail!("serializing to {:?} is not supported", encoding),
        };

        if self.opts.newline {
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn serialize_yaml<W>(writer: &mut W, value: Value) -> Result<()>
where
    W: std::io::Write,
{
    Ok(serde_yaml::to_writer(writer, &value)?)
}

fn serialize_json<W>(writer: &mut W, value: Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    if opts.pretty {
        serde_json::to_writer_pretty(writer, &value)?;
    } else {
        serde_json::to_writer(writer, &value)?;
    }

    Ok(())
}

fn serialize_ron<W>(writer: &mut W, value: Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    if opts.pretty {
        ron::ser::to_writer_pretty(writer, &value, ron::ser::PrettyConfig::default())?
    } else {
        ron::ser::to_writer(writer, &value)?
    }

    Ok(())
}

fn serialize_toml<W>(writer: &mut W, value: Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    let s = if opts.pretty {
        toml::ser::to_string_pretty(&value)?
    } else {
        toml::ser::to_string(&value)?
    };

    Ok(writer.write_all(s.as_bytes())?)
}

fn serialize_csv<W>(writer: &mut W, delimiter: u8, value: Value) -> Result<()>
where
    W: std::io::Write,
{
    if !value.is_array() {
        bail!("serializing to CSV requires the input data to be an array",)
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(writer);

    for row in value.as_array().unwrap() {
        csv_writer.serialize(row)?;
    }

    Ok(())
}
