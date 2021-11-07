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
    /// Optional seprator to join text output with.
    pub text_join_separator: Option<String>,
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
/// let writer = std::io::stdout();
/// let mut serializer = SerializerBuilder::new()
///     .pretty(true)
///     .newline(true)
///     .build(writer);
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

    /// Sets a custom separator to join text output with.
    pub fn text_join_separator<S>(&mut self, sep: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.opts.text_join_separator = Some(sep.as_ref().to_owned());
        self
    }

    /// Builds the `Serializer` for the given writer.
    pub fn build<W>(&self, writer: W) -> Serializer<W>
    where
        W: std::io::Write,
    {
        Serializer::with_options(writer, self.opts.clone())
    }
}

/// A `Serializer` can serialize a `Value` into an encoded byte stream.
pub struct Serializer<W> {
    writer: W,
    opts: SerializeOptions,
}

impl<W> Serializer<W>
where
    W: std::io::Write,
{
    /// Creates a new `Serializer` for writer with default options.
    pub fn new(writer: W) -> Self {
        Self::with_options(writer, Default::default())
    }

    /// Creates a new `Serializer` for writer with options.
    pub fn with_options(writer: W, opts: SerializeOptions) -> Self {
        Self { writer, opts }
    }

    /// Serializes the given `Value` and writes the encoded data to the writer.
    ///
    /// ## Example
    ///
    /// ```
    /// use dts::{ser::SerializerBuilder, Encoding};
    /// use serde_json::json;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let mut buf = Vec::new();
    /// let mut ser = SerializerBuilder::new().build(&mut buf);
    /// ser.serialize(Encoding::Json, &json!(["foo"]))?;
    ///
    /// assert_eq!(&buf, r#"["foo"]"#.as_bytes());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn serialize(&mut self, encoding: Encoding, value: &Value) -> Result<()>
    where
        W: std::io::Write,
    {
        match encoding {
            Encoding::Yaml => self.serialize_yaml(value)?,
            Encoding::Json | Encoding::Json5 => self.serialize_json(value)?,
            Encoding::Ron => self.serialize_ron(value)?,
            Encoding::Toml => self.serialize_toml(value)?,
            Encoding::Csv => self.serialize_csv(value)?,
            Encoding::Pickle => self.serialize_pickle(value)?,
            Encoding::QueryString => self.serialize_query_string(value)?,
            Encoding::Xml => self.serialize_xml(value)?,
            Encoding::Text => self.serialize_text(value)?,
            encoding => return Err(Error::SerializeUnsupported(encoding)),
        };

        if self.opts.newline {
            self.writer.write_all(b"\n")?;
        }

        Ok(())
    }

    fn serialize_yaml(&mut self, value: &Value) -> Result<()> {
        Ok(serde_yaml::to_writer(&mut self.writer, value)?)
    }

    fn serialize_json(&mut self, value: &Value) -> Result<()> {
        if self.opts.pretty {
            serde_json::to_writer_pretty(&mut self.writer, value)?
        } else {
            serde_json::to_writer(&mut self.writer, value)?
        }

        Ok(())
    }

    fn serialize_ron(&mut self, value: &Value) -> Result<()> {
        if self.opts.pretty {
            ron::ser::to_writer_pretty(&mut self.writer, value, Default::default())?
        } else {
            ron::ser::to_writer(&mut self.writer, value)?
        }

        Ok(())
    }

    fn serialize_toml(&mut self, value: &Value) -> Result<()> {
        let s = if self.opts.pretty {
            toml::ser::to_string_pretty(value)?
        } else {
            toml::ser::to_string(value)?
        };

        Ok(self.writer.write_all(s.as_bytes())?)
    }

    fn serialize_csv(&mut self, value: &Value) -> Result<()> {
        let value = value.as_array().ok_or_else(|| {
            Error::new("serializing to csv requires the input data to be an array")
        })?;

        // Because individual row items may produce errors during serialization because they are of
        // unexpected type, write into a buffer first and only flush out to the writer only if
        // serialization of all rows succeeded. This avoids writing out partial data.
        let mut buf = Vec::new();
        {
            let mut csv_writer = csv::WriterBuilder::new()
                .delimiter(self.opts.csv_delimiter.unwrap_or(b','))
                .from_writer(&mut buf);

            let mut headers: Option<Vec<&String>> = None;

            // Empty string value which will be referenced for missing fields.
            let empty = Value::String(String::new());

            for (i, row) in value.iter().enumerate() {
                if !self.opts.keys_as_csv_headers {
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

        Ok(self.writer.write_all(&buf)?)
    }

    fn serialize_pickle(&mut self, value: &Value) -> Result<()> {
        Ok(serde_pickle::to_writer(
            &mut self.writer,
            value,
            Default::default(),
        )?)
    }

    fn serialize_query_string(&mut self, value: &Value) -> Result<()> {
        Ok(serde_qs::to_writer(value, &mut self.writer)?)
    }

    fn serialize_xml(&mut self, value: &Value) -> Result<()> {
        Ok(serde_xml_rs::to_writer(&mut self.writer, value)?)
    }

    fn serialize_text(&mut self, value: &Value) -> Result<()> {
        let sep = self
            .opts
            .text_join_separator
            .clone()
            .unwrap_or_else(|| String::from("\n"));

        let text = value
            .as_array()
            .ok_or_else(|| {
                Error::new("serializing to text requires the input data to be an array")
            })?
            .iter()
            .map(|value| {
                match value {
                    // Use strings directly to prevent quoting
                    Value::String(s) => Ok(s.to_string()),
                    other => Ok(serde_json::to_string(other)?),
                }
            })
            .collect::<Result<Vec<String>>>()?
            .join(&sep);

        Ok(self.writer.write_all(text.as_bytes())?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_serialize_json() {
        let mut buf = Vec::new();
        let mut ser = Serializer::new(&mut buf);
        ser.serialize(Encoding::Json, &json!(["one", "two"]))
            .unwrap();
        assert_eq!(&buf, "[\"one\",\"two\"]".as_bytes());

        buf.clear();

        let mut ser = SerializerBuilder::new().pretty(true).build(&mut buf);
        ser.serialize(Encoding::Json, &json!(["one", "two"]))
            .unwrap();
        assert_eq!(&buf, "[\n  \"one\",\n  \"two\"\n]".as_bytes());
    }

    #[test]
    fn test_serialize_csv() {
        let mut buf = Vec::new();
        let mut ser = Serializer::new(&mut buf);
        ser.serialize(Encoding::Csv, &json!([["one", "two"], ["three", "four"]]))
            .unwrap();
        assert_eq!(&buf, "one,two\nthree,four\n".as_bytes());

        buf.clear();

        let mut ser = SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(&mut buf);
        ser.serialize(
            Encoding::Csv,
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
        let mut ser = Serializer::new(&mut buf);
        assert!(ser.serialize(Encoding::Csv, &json!("non-array")).is_err());
        assert!(ser
            .serialize(Encoding::Csv, &json!([{"non-array": "row"}]))
            .is_err());

        let mut ser = SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(&mut buf);
        assert!(ser
            .serialize(Encoding::Csv, &json!([["non-object-row"]]))
            .is_err());
    }

    #[test]
    fn test_serialize_text() {
        let mut buf = Vec::new();
        let mut ser = Serializer::new(&mut buf);
        ser.serialize(Encoding::Text, &json!(["one", "two"]))
            .unwrap();
        assert_eq!(&buf, "one\ntwo".as_bytes());

        let mut buf = Vec::new();
        let mut ser = Serializer::new(&mut buf);
        ser.serialize(Encoding::Text, &json!([{"foo": "bar"}, "baz"]))
            .unwrap();
        assert_eq!(&buf, "{\"foo\":\"bar\"}\nbaz".as_bytes());
    }

    #[test]
    fn test_serialize_text_error() {
        let mut buf = Vec::new();
        let mut ser = Serializer::new(&mut buf);
        assert!(ser
            .serialize(Encoding::Text, &json!({"foo": "bar"}))
            .is_err());
    }
}
