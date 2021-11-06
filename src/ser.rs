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
    /// Optional custom delimiter for CSV output.
    pub csv_delimiter: Option<u8>,
}

impl SerializeOptions {
    /// Creates new `SerializeOptions`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A `SerializerBuilder` can be used to build a `Serializer` with certain
/// `SerializeOptions`.
///
/// ## Example
///
/// ```
/// use dts::{ser::SerializerBuilder, Encoding};
///
/// let serializer = SerializerBuilder::new()
///     .pretty(true)
///     .newline(true)
///     .build(Encoding::Json);
/// ```
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

    /// Sets a custom CSV delimiter.
    pub fn csv_delimiter(&mut self, delim: u8) -> &mut Self {
        self.opts.csv_delimiter = Some(delim);
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
    ///
    /// ## Example
    ///
    /// ```
    /// use dts::{ser::SerializerBuilder, Encoding};
    /// use serde_json::json;
    ///
    /// let ser = SerializerBuilder::new().build(Encoding::Json);
    ///
    /// let mut buf = Vec::new();
    /// ser.serialize(&mut buf, &json!(["foo"])).unwrap();
    ///
    /// assert_eq!(&buf, r#"["foo"]"#.as_bytes());
    /// ```
    pub fn serialize<W>(&self, writer: &mut W, value: &Value) -> Result<()>
    where
        W: std::io::Write,
    {
        match &self.encoding {
            Encoding::Yaml => serialize_yaml(writer, value)?,
            Encoding::Json | Encoding::Json5 => serialize_json(writer, value, &self.opts)?,
            Encoding::Ron => serialize_ron(writer, value, &self.opts)?,
            Encoding::Toml => serialize_toml(writer, value, &self.opts)?,
            Encoding::Csv => serialize_csv(writer, value, &self.opts)?,
            Encoding::Pickle => serialize_pickle(writer, value)?,
            Encoding::QueryString => serialize_query_string(writer, value)?,
            Encoding::Xml => serialize_xml(writer, value)?,
            Encoding::Text => serialize_text(writer, value)?,
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

fn serialize_csv<W>(writer: &mut W, value: &Value, opts: &SerializeOptions) -> Result<()>
where
    W: std::io::Write,
{
    let value = value
        .as_array()
        .ok_or_else(|| Error::new("serializing to csv requires the input data to be an array"))?;

    let mut buf = Vec::new();
    {
        let mut csv_writer = csv::WriterBuilder::new()
            .delimiter(opts.csv_delimiter.unwrap_or(b','))
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

fn serialize_text<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: std::io::Write,
{
    let text = value
        .as_array()
        .ok_or_else(|| Error::new("serializing to text requires the input data to be an array"))?
        .iter()
        .map(|value| {
            match value {
                // Use strings directly to prevent quoting
                Value::String(s) => Ok(s.to_string()),
                other => Ok(serde_json::to_string(other)?),
            }
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n");

    Ok(writer.write_all(text.as_bytes())?)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_serialize_json() {
        let mut buf = Vec::new();

        let ser = SerializerBuilder::new().build(Encoding::Json);
        ser.serialize(&mut buf, &json!(["one", "two"])).unwrap();
        assert_eq!(&buf, "[\"one\",\"two\"]".as_bytes());

        buf.clear();

        let ser = SerializerBuilder::new().pretty(true).build(Encoding::Json);
        ser.serialize(&mut buf, &json!(["one", "two"])).unwrap();
        assert_eq!(&buf, "[\n  \"one\",\n  \"two\"\n]".as_bytes());
    }

    #[test]
    fn test_serialize_csv() {
        let mut buf = Vec::new();

        let ser = SerializerBuilder::new().build(Encoding::Csv);
        ser.serialize(&mut buf, &json!([["one", "two"], ["three", "four"]]))
            .unwrap();
        assert_eq!(&buf, "one,two\nthree,four\n".as_bytes());

        buf.clear();

        let ser = SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(Encoding::Csv);
        ser.serialize(
            &mut buf,
            &json!([
                {"one": "val1", "two": "val2"},
                {"one": "val3", "three": "val4"},
                {"two": "val5"}
            ]),
        )
        .unwrap();
        assert_eq!(&buf, "one,two\nval1,val2\nval3,\n,val5\n".as_bytes());
    }

    #[test]
    fn test_serialize_csv_errors() {
        let mut buf = Vec::new();

        let ser = SerializerBuilder::new().build(Encoding::Csv);
        assert!(ser.serialize(&mut buf, &json!("non-array")).is_err());
        assert!(ser
            .serialize(&mut buf, &json!([{"non-array": "row"}]))
            .is_err());

        let ser = SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(Encoding::Csv);
        assert!(ser
            .serialize(&mut buf, &json!([["non-object-row"]]))
            .is_err());
    }

    #[test]
    fn test_serialize_text() {
        let mut buf = Vec::new();

        let ser = SerializerBuilder::new().build(Encoding::Text);
        ser.serialize(&mut buf, &json!(["one", "two"])).unwrap();
        assert_eq!(&buf, "one\ntwo".as_bytes());

        buf.clear();

        ser.serialize(&mut buf, &json!([{"foo": "bar"}, "baz"]))
            .unwrap();
        assert_eq!(&buf, "{\"foo\":\"bar\"}\nbaz".as_bytes());
    }

    #[test]
    fn test_serialize_text_error() {
        let mut buf = Vec::new();

        let ser = SerializerBuilder::new().build(Encoding::Text);
        assert!(ser.serialize(&mut buf, &json!({"foo": "bar"})).is_err());
    }
}
