//! This module provides a `Serializer` which supports serializing values into various output
//! encodings.

use crate::{Encoding, Error, Result, Value};

/// Options for the `Serializer`. The options are context specific and may only be honored when
/// serializing into a certain `Encoding`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SerializeOptions {
    /// Pretty print the serialized data if supported by the encoding.
    pub pretty: bool,
    /// Append a trailing newline to the serialized data.
    pub newline: bool,
    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will
    /// be matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    pub keys_as_csv_headers: bool,
}

impl SerializeOptions {
    /// Creates new `SerializeOptions`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A `SerializerBuilder` can be used to build a `Serializer` with certain
/// `SerializeOptions`.
#[derive(Debug, Default, Clone)]
pub struct SerializerBuilder {
    opts: SerializeOptions,
}

impl SerializerBuilder {
    /// Creates a new `SerializerBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pretty print the serialized data if supported by the encoding.
    pub fn pretty(&mut self, yes: bool) -> &mut Self {
        self.opts.pretty = yes;
        self
    }

    /// Append a trailing newline to the serialized data.
    pub fn newline(&mut self, yes: bool) -> &mut Self {
        self.opts.newline = yes;
        self
    }

    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will
    /// be matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    pub fn keys_as_csv_headers(&mut self, yes: bool) -> &mut Self {
        self.opts.keys_as_csv_headers = yes;
        self
    }

    /// Builds the `Serializer` for the given `Encoding`.
    pub fn build(&self, encoding: Encoding) -> Serializer {
        Serializer::new(encoding, self.opts.clone())
    }
}

/// A `Serializer` can serialize a `Value` into an encoded byte stream.
pub struct Serializer {
    encoding: Encoding,
    opts: SerializeOptions,
}

impl Serializer {
    /// Creates a new `Serializer` for `Encoding` with options.
    pub fn new(encoding: Encoding, opts: SerializeOptions) -> Self {
        Self { encoding, opts }
    }

    /// Serializes the given `Value` and writes the encoded data to the writer.
    pub fn serialize<W>(&self, writer: &mut W, value: &Value) -> Result<()>
    where
        W: std::io::Write,
    {
        match &self.encoding {
            Encoding::Yaml => serialize_yaml(writer, value)?,
            Encoding::Json | Encoding::Json5 => serialize_json(writer, value, &self.opts)?,
            Encoding::Ron => serialize_ron(writer, value, &self.opts)?,
            Encoding::Toml => serialize_toml(writer, value, &self.opts)?,
            Encoding::Csv => serialize_csv(writer, b',', value, &self.opts)?,
            Encoding::Tsv => serialize_csv(writer, b'\t', value, &self.opts)?,
            Encoding::Pickle => serialize_pickle(writer, value)?,
            Encoding::QueryString => serialize_query_string(writer, value)?,
            Encoding::Xml => serialize_xml(writer, value)?,
            &encoding => return Err(Error::SerializeUnsupported(encoding)),
        };

        if self.opts.newline {
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn serialize_yaml<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: std::io::Write,
{
    Ok(serde_yaml::to_writer(writer, value)?)
}

fn serialize_json<W>(writer: &mut W, value: &Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    if opts.pretty {
        serde_json::to_writer_pretty(writer, value)?
    } else {
        serde_json::to_writer(writer, value)?
    }

    Ok(())
}

fn serialize_ron<W>(writer: &mut W, value: &Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    if opts.pretty {
        ron::ser::to_writer_pretty(writer, value, Default::default())?
    } else {
        ron::ser::to_writer(writer, value)?
    }

    Ok(())
}

fn serialize_toml<W>(writer: &mut W, value: &Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    let s = if opts.pretty {
        toml::ser::to_string_pretty(value)?
    } else {
        toml::ser::to_string(value)?
    };

    Ok(writer.write_all(s.as_bytes())?)
}

fn serialize_csv<W>(
    writer: &mut W,
    delimiter: u8,
    value: &Value,
    opts: &SerializeOptions,
) -> Result<()>
where
    W: std::io::Write,
{
    let value = value
        .as_array()
        .ok_or_else(|| Error::new("serializing to csv requires the input data to be an array"))?;

    let mut buf = Vec::new();
    {
        let mut csv_writer = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(&mut buf);

        let mut headers: Option<Vec<&String>> = None;

        // Empty string value which will be referenced for missing fields.
        let empty = Value::String("".into());

        for (i, row) in value.iter().enumerate() {
            if !opts.keys_as_csv_headers {
                let row_data = row
                    .as_array()
                    .ok_or_else(|| Error::at_row_index(i, "array expected"))?;

                csv_writer.serialize(row_data)?;
            } else {
                let row = row
                    .as_object()
                    .ok_or_else(|| Error::at_row_index(i, "object expected"))?;

                // The first row dictates the header fields.
                if headers.is_none() {
                    let header_data = row.keys().collect();
                    csv_writer.serialize(&header_data)?;
                    headers = Some(header_data);
                }

                let row_data = headers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|&header| row.get(header).or(Some(&empty)))
                    .collect::<Vec<_>>();

                csv_writer.serialize(row_data)?;
            }
        }
    }

    Ok(writer.write_all(&buf)?)
}

fn serialize_pickle<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: std::io::Write,
{
    Ok(serde_pickle::to_writer(writer, value, Default::default())?)
}

fn serialize_query_string<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: std::io::Write,
{
    Ok(serde_qs::to_writer(value, writer)?)
}

fn serialize_xml<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: std::io::Write,
{
    Ok(serde_xml_rs::to_writer(writer, value)?)
}
