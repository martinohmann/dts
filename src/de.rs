//! This module provides a `Deserializer` which supports deserializing input data with various
//! encodings into a `Value`.

use crate::{Encoding, Result, key::expand_keys, parsers::gron};
use hcl::eval::Evaluate;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Map, Value};

/// Options for the `Deserializer`. The options are context specific and may only be honored when
/// deserializing from a certain `Encoding`.
#[derive(Debug, Default, Clone)]
pub struct DeserializeOptions {
    /// Indicates that an input CSV does not include a header line. If `false`, the first line is
    /// discarded.
    pub csv_without_headers: bool,
    /// Indicates that the header fields of an input CSV should be used as keys for each row's
    /// columns. This means that the deserialized row data will be of type object. Otherwise row
    /// data will be of type array.
    pub csv_headers_as_keys: bool,
    /// Optional custom delimiter for CSV input.
    pub csv_delimiter: Option<u8>,
    /// Optional regex pattern to split text input at.
    pub text_split_pattern: Option<Regex>,
    /// Simplify input if the encoding supports it.
    pub simplify: bool,
}

impl DeserializeOptions {
    /// Creates new `DeserializeOptions`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A `DeserializerBuilder` can be used to build a `Deserializer` with certain
/// `DeserializeOptions`.
///
/// ## Example
///
/// ```
/// use dts::{de::DeserializerBuilder, Encoding};
///
/// let buf = r#"["foo"]"#.as_bytes();
///
/// let deserializer = DeserializerBuilder::new()
///     .csv_delimiter(b'\t')
///     .build(buf);
/// ```
#[derive(Debug, Default, Clone)]
pub struct DeserializerBuilder {
    opts: DeserializeOptions,
}

impl DeserializerBuilder {
    /// Creates a new `DeserializerBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Indicates that an input CSV does not include a header line. If `false`, the first line is
    /// discarded.
    pub fn csv_without_headers(&mut self, yes: bool) -> &mut Self {
        self.opts.csv_without_headers = yes;
        self
    }

    /// Indicates that the header fields of an input CSV should be used as keys for each row's
    /// columns. This means that the deserialized row data will be of type object. Otherwise row
    /// data will be of type array.
    pub fn csv_headers_as_keys(&mut self, yes: bool) -> &mut Self {
        self.opts.csv_headers_as_keys = yes;
        self
    }

    /// Sets a custom CSV delimiter.
    pub fn csv_delimiter(&mut self, delim: u8) -> &mut Self {
        self.opts.csv_delimiter = Some(delim);
        self
    }

    /// Sets regex pattern to split text at.
    pub fn text_split_pattern(&mut self, pattern: Regex) -> &mut Self {
        self.opts.text_split_pattern = Some(pattern);
        self
    }

    /// Simplify input if the encoding supports it.
    pub fn simplifiy(&mut self, yes: bool) -> &mut Self {
        self.opts.simplify = yes;
        self
    }

    /// Builds the `Deserializer` for the given reader.
    pub fn build<R>(&self, reader: R) -> Deserializer<R>
    where
        R: std::io::Read,
    {
        Deserializer::with_options(reader, self.opts.clone())
    }
}

/// A `Deserializer` can deserialize input data from a reader into a `Value`.
pub struct Deserializer<R> {
    reader: R,
    opts: DeserializeOptions,
}

impl<R> Deserializer<R>
where
    R: std::io::Read,
{
    /// Creates a new `Deserializer` for reader with default options.
    pub fn new(reader: R) -> Self {
        Self::with_options(reader, Default::default())
    }

    /// Creates a new `Deserializer` for reader with options.
    pub fn with_options(reader: R, opts: DeserializeOptions) -> Self {
        Self { reader, opts }
    }

    /// Reads input data from the given reader and deserializes it in a `Value`.
    ///
    /// ## Example
    ///
    /// ```
    /// use dts::{de::DeserializerBuilder, Encoding};
    /// use serde_json::json;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = r#"["foo"]"#.as_bytes();
    ///
    /// let mut de = DeserializerBuilder::new().build(buf);
    /// let value = de.deserialize(Encoding::Json)?;
    ///
    /// assert_eq!(value, json!(["foo"]));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn deserialize(&mut self, encoding: Encoding) -> Result<Value> {
        match encoding {
            Encoding::Yaml => self.deserialize_yaml(),
            Encoding::Json => self.deserialize_json(),
            Encoding::Toml => self.deserialize_toml(),
            Encoding::Json5 => self.deserialize_json5(),
            Encoding::Csv => self.deserialize_csv(),
            Encoding::QueryString => self.deserialize_query_string(),
            Encoding::Xml => self.deserialize_xml(),
            Encoding::Text => self.deserialize_text(),
            Encoding::Gron => self.deserialize_gron(),
            Encoding::Hcl => self.deserialize_hcl(),
        }
    }

    fn deserialize_yaml(&mut self) -> Result<Value> {
        let mut values = serde_yaml::Deserializer::from_reader(&mut self.reader)
            .map(Value::deserialize)
            .collect::<Result<Vec<_>, _>>()?;

        // If this was not multi-document YAML, just take the first document's value without
        // wrapping it into an array.
        if values.len() == 1 {
            Ok(values.swap_remove(0))
        } else {
            Ok(Value::Array(values))
        }
    }

    fn deserialize_json(&mut self) -> Result<Value> {
        Ok(serde_json::from_reader(&mut self.reader)?)
    }

    fn deserialize_toml(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;
        Ok(toml::from_str(&s)?)
    }

    fn deserialize_json5(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;
        Ok(json5::from_str(&s)?)
    }

    fn deserialize_csv(&mut self) -> Result<Value> {
        let keep_first_line = self.opts.csv_without_headers || self.opts.csv_headers_as_keys;

        let mut csv_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .has_headers(!keep_first_line)
            .delimiter(self.opts.csv_delimiter.unwrap_or(b','))
            .from_reader(&mut self.reader);

        let mut iter = csv_reader.deserialize();

        let value = if self.opts.csv_headers_as_keys {
            match iter.next() {
                Some(headers) => {
                    let headers: Vec<String> = headers?;

                    Value::Array(
                        iter.map(|record| {
                            Ok(headers.iter().cloned().zip(record?.into_iter()).collect())
                        })
                        .collect::<Result<_>>()?,
                    )
                }
                None => Value::Array(Vec::new()),
            }
        } else {
            Value::Array(
                iter.map(|v| Ok(serde_json::to_value(v?)?))
                    .collect::<Result<_>>()?,
            )
        };

        Ok(value)
    }

    fn deserialize_query_string(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;
        Ok(Value::Object(serde_qs::from_str(&s)?))
    }

    fn deserialize_xml(&mut self) -> Result<Value> {
        Ok(serde_xml_rs::from_reader(&mut self.reader)?)
    }

    fn deserialize_text(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;

        let pattern = match &self.opts.text_split_pattern {
            Some(pattern) => pattern.clone(),
            None => Regex::new("\n").unwrap(),
        };

        Ok(Value::Array(
            pattern
                .split(&s)
                .map(serde_json::to_value)
                .collect::<Result<_, serde_json::Error>>()?,
        ))
    }

    fn deserialize_gron(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;

        let map = gron::parse(&s)?
            .iter()
            .map(|statement| {
                Ok((
                    statement.path().to_owned(),
                    serde_json::from_str(statement.value())?,
                ))
            })
            .collect::<Result<Map<_, _>>>()?;

        Ok(expand_keys(Value::Object(map)))
    }

    fn deserialize_hcl(&mut self) -> Result<Value> {
        let value = if self.opts.simplify {
            let mut body: hcl::Body = hcl::from_reader(&mut self.reader)?;
            let ctx = hcl::eval::Context::new();
            let _ = body.evaluate_in_place(&ctx);
            hcl::from_body(body)?
        } else {
            hcl::from_reader(&mut self.reader)?
        };

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[track_caller]
    fn assert_builder_deserializes_to(
        builder: &mut DeserializerBuilder,
        encoding: Encoding,
        input: &str,
        expected: Value,
    ) {
        let mut de = builder.build(input.as_bytes());
        let value = de.deserialize(encoding).unwrap();
        assert_eq!(value, expected);
    }

    #[track_caller]
    fn assert_deserializes_to(encoding: Encoding, input: &str, expected: Value) {
        assert_builder_deserializes_to(&mut DeserializerBuilder::new(), encoding, input, expected);
    }

    #[test]
    fn test_deserialize_yaml() {
        assert_deserializes_to(Encoding::Yaml, "---\nfoo: bar", json!({"foo": "bar"}));
        assert_deserializes_to(
            Encoding::Yaml,
            "---\nfoo: bar\n---\nbaz: qux",
            json!([{"foo": "bar"}, {"baz": "qux"}]),
        );
    }

    #[test]
    fn test_deserialize_csv() {
        assert_deserializes_to(
            Encoding::Csv,
            "header1,header2\ncol1,col2",
            json!([["col1", "col2"]]),
        );
        assert_builder_deserializes_to(
            &mut DeserializerBuilder::new().csv_without_headers(true),
            Encoding::Csv,
            "row1col1,row1col2\nrow2col1,row2col2",
            json!([["row1col1", "row1col2"], ["row2col1", "row2col2"]]),
        );
        assert_builder_deserializes_to(
            &mut DeserializerBuilder::new().csv_headers_as_keys(true),
            Encoding::Csv,
            "header1,header2\nrow1col1,row1col2\nrow2col1,row2col2",
            json!([{"header1":"row1col1", "header2":"row1col2"}, {"header1":"row2col1", "header2":"row2col2"}]),
        );
        assert_builder_deserializes_to(
            &mut DeserializerBuilder::new().csv_delimiter(b'|'),
            Encoding::Csv,
            "header1|header2\ncol1|col2",
            json!([["col1", "col2"]]),
        );
    }

    #[test]
    fn test_deserialize_text() {
        assert_deserializes_to(
            Encoding::Text,
            "one\ntwo\nthree\n",
            json!(["one", "two", "three", ""]),
        );
        assert_deserializes_to(Encoding::Text, "", json!([""]));
    }
}
