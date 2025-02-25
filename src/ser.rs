//! This module provides a `Serializer` which supports serializing values into various output
//! encodings.

use crate::{Encoding, Error, Result, key::flatten_keys, value::ValueExt};
use serde_json::Value;
use std::fmt::Write;

/// Options for the `Serializer`. The options are context specific and may only be honored when
/// serializing into a certain `Encoding`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SerializeOptions {
    /// Emit output data in a compact format. This will disable pretty printing for encodings that
    /// support it.
    pub compact: bool,
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
    /// Treat output arrays as multiple YAML documents.
    pub multi_doc_yaml: bool,
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

    /// Emit output data in a compact format. This will disable pretty printing for encodings that
    /// support it.
    pub fn compact(&mut self, yes: bool) -> &mut Self {
        self.opts.compact = yes;
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

    /// Treat output arrays as multiple YAML documents.
    pub fn multi_doc_yaml(&mut self, yes: bool) -> &mut Self {
        self.opts.multi_doc_yaml = yes;
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
    /// let mut ser = SerializerBuilder::new().compact(true).build(&mut buf);
    /// ser.serialize(Encoding::Json, json!(["foo"]))?;
    ///
    /// assert_eq!(&buf, r#"["foo"]"#.as_bytes());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn serialize(&mut self, encoding: Encoding, value: Value) -> Result<()> {
        match encoding {
            Encoding::Yaml => self.serialize_yaml(value)?,
            Encoding::Json => self.serialize_json(value)?,
            Encoding::Toml => self.serialize_toml(value)?,
            Encoding::Csv => self.serialize_csv(value)?,
            Encoding::QueryString => self.serialize_query_string(value)?,
            Encoding::Xml => self.serialize_xml(value)?,
            Encoding::Text => self.serialize_text(value)?,
            Encoding::Gron => self.serialize_gron(value)?,
            Encoding::Hcl => self.serialize_hcl(value)?,
            encoding => return Err(Error::UnsupportedEncoding(encoding)),
        };

        if self.opts.newline {
            self.writer.write_all(b"\n")?;
        }

        Ok(())
    }

    fn serialize_yaml(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Array(array) if self.opts.multi_doc_yaml => array
                .into_iter()
                .try_for_each(|document| self.serialize_yaml_document(&document)),
            value => self.serialize_yaml_document(&value),
        }
    }

    fn serialize_yaml_document(&mut self, value: &Value) -> Result<()> {
        self.writer.write_all(b"---\n")?;
        serde_yaml::to_writer(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_json(&mut self, value: Value) -> Result<()> {
        if self.opts.compact {
            serde_json::to_writer(&mut self.writer, &value)?
        } else {
            serde_json::to_writer_pretty(&mut self.writer, &value)?
        }

        Ok(())
    }

    fn serialize_toml(&mut self, value: Value) -> Result<()> {
        let value = toml::Value::try_from(value)?;

        let s = if self.opts.compact {
            toml::ser::to_string(&value)?
        } else {
            toml::ser::to_string_pretty(&value)?
        };

        Ok(self.writer.write_all(s.as_bytes())?)
    }

    fn serialize_csv(&mut self, value: Value) -> Result<()> {
        // Because individual row items may produce errors during serialization because they are of
        // unexpected type, write into a buffer first and only flush out to the writer only if
        // serialization of all rows succeeded. This avoids writing out partial data.
        let mut buf = Vec::new();
        {
            let mut csv_writer = csv::WriterBuilder::new()
                .delimiter(self.opts.csv_delimiter.unwrap_or(b','))
                .from_writer(&mut buf);

            let mut headers: Option<Vec<String>> = None;
            let empty_value = Value::String("".into());

            for row in value.into_array().into_iter() {
                let row_data = if !self.opts.keys_as_csv_headers {
                    row.into_array()
                        .into_iter()
                        .map(Value::into_string)
                        .collect::<Vec<_>>()
                } else {
                    let row = row.into_object("csv");

                    // The first row dictates the header fields.
                    if headers.is_none() {
                        let header_data = row.keys().cloned().collect();
                        csv_writer.serialize(&header_data)?;
                        headers = Some(header_data);
                    }

                    headers
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|header| row.get(header).unwrap_or(&empty_value))
                        .cloned()
                        .map(Value::into_string)
                        .collect::<Vec<_>>()
                };

                csv_writer.serialize(row_data)?;
            }
        }

        Ok(self.writer.write_all(&buf)?)
    }

    fn serialize_query_string(&mut self, value: Value) -> Result<()> {
        Ok(serde_qs::to_writer(&value, &mut self.writer)?)
    }

    fn serialize_xml(&mut self, value: Value) -> Result<()> {
        Ok(serde_xml_rs::to_writer(&mut self.writer, &value)?)
    }

    fn serialize_text(&mut self, value: Value) -> Result<()> {
        let sep = self
            .opts
            .text_join_separator
            .clone()
            .unwrap_or_else(|| String::from('\n'));

        let text = value
            .into_array()
            .into_iter()
            .map(Value::into_string)
            .collect::<Vec<String>>()
            .join(&sep);

        Ok(self.writer.write_all(text.as_bytes())?)
    }

    fn serialize_gron(&mut self, value: Value) -> Result<()> {
        let output = flatten_keys(value, "json")
            .as_object()
            .unwrap()
            .into_iter()
            .fold(String::new(), |mut output, (k, v)| {
                let _ = writeln!(output, "{k} = {v};");
                output
            });

        Ok(self.writer.write_all(output.as_bytes())?)
    }

    fn serialize_hcl(&mut self, value: Value) -> Result<()> {
        if self.opts.compact {
            let fmt = hcl::format::Formatter::builder()
                .compact(self.opts.compact)
                .build(&mut self.writer);
            let mut ser = hcl::ser::Serializer::with_formatter(fmt);
            ser.serialize(&value)?;
        } else {
            hcl::to_writer(&mut self.writer, &value)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::str;

    #[track_caller]
    fn assert_serializes_to(encoding: Encoding, value: Value, expected: &str) {
        assert_builder_serializes_to(&mut SerializerBuilder::new(), encoding, value, expected)
    }

    #[track_caller]
    fn assert_builder_serializes_to(
        builder: &mut SerializerBuilder,
        encoding: Encoding,
        value: Value,
        expected: &str,
    ) {
        let mut buf = Vec::new();
        let mut ser = builder.build(&mut buf);

        ser.serialize(encoding, value).unwrap();
        assert_eq!(str::from_utf8(&buf).unwrap(), expected);
    }

    #[test]
    fn test_serialize_json() {
        assert_builder_serializes_to(
            &mut SerializerBuilder::new().compact(true),
            Encoding::Json,
            json!(["one", "two"]),
            "[\"one\",\"two\"]",
        );
        assert_serializes_to(
            Encoding::Json,
            json!(["one", "two"]),
            "[\n  \"one\",\n  \"two\"\n]",
        );
    }

    #[test]
    fn test_serialize_csv() {
        assert_serializes_to(
            Encoding::Csv,
            json!([["one", "two"], ["three", "four"]]),
            "one,two\nthree,four\n",
        );
        assert_builder_serializes_to(
            &mut SerializerBuilder::new().keys_as_csv_headers(true),
            Encoding::Csv,
            json!([
                {"one": "val1", "two": "val2"},
                {"one": "val3", "three": "val4"},
                {"two": "val5"}
            ]),
            "one,two\nval1,val2\nval3,\n,val5\n",
        );
        assert_builder_serializes_to(
            &mut SerializerBuilder::new().keys_as_csv_headers(true),
            Encoding::Csv,
            json!({"one": "val1", "two": "val2"}),
            "one,two\nval1,val2\n",
        );
        assert_serializes_to(Encoding::Csv, json!("non-array"), "non-array\n");
        assert_serializes_to(
            Encoding::Csv,
            json!([{"non-array": "row"}]),
            "\"{\"\"non-array\"\":\"\"row\"\"}\"\n",
        );
        assert_builder_serializes_to(
            &mut SerializerBuilder::new().keys_as_csv_headers(true),
            Encoding::Csv,
            json!([["non-object-row"]]),
            "csv\n\"[\"\"non-object-row\"\"]\"\n",
        );
    }

    #[test]
    fn test_serialize_text() {
        assert_serializes_to(Encoding::Text, json!(["one", "two"]), "one\ntwo");
        assert_serializes_to(
            Encoding::Text,
            json!([{"foo": "bar"}, "baz"]),
            "{\"foo\":\"bar\"}\nbaz",
        );
        assert_serializes_to(Encoding::Text, json!({"foo": "bar"}), "{\"foo\":\"bar\"}");
    }

    #[test]
    fn test_serialize_hcl() {
        assert_serializes_to(Encoding::Hcl, json!([{"foo": "bar"}]), "foo = \"bar\"\n");
        assert_serializes_to(
            Encoding::Hcl,
            json!({"foo": "bar", "bar": 2}),
            "foo = \"bar\"\nbar = 2\n",
        );
    }
}
