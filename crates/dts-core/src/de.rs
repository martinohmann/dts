//! This module provides a `Deserializer` which supports deserializing input data with various
//! encodings into a `Value`.

use crate::parsers::gron::Statements as GronStatements;
use crate::{Encoding, Result, Value, ValueExt};
use regex::Regex;
use serde::Deserialize;
use serde_json::Map;

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
/// use dts_core::{de::DeserializerBuilder, Encoding};
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
    /// use dts_core::{de::DeserializerBuilder, Encoding};
    /// use serde_json::json;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = r#"["foo"]"#.as_bytes();
    ///
    /// let mut de = DeserializerBuilder::new().build(buf);
    /// let value = de.deserialize(Encoding::JSON)?;
    ///
    /// assert_eq!(value, json!(["foo"]));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn deserialize(&mut self, encoding: Encoding) -> Result<Value> {
        match encoding {
            Encoding::YAML => self.deserialize_yaml(),
            Encoding::JSON => self.deserialize_json(),
            Encoding::TOML => self.deserialize_toml(),
            Encoding::JSON5 => self.deserialize_json5(),
            Encoding::CSV => self.deserialize_csv(),
            Encoding::QueryString => self.deserialize_query_string(),
            Encoding::XML => self.deserialize_xml(),
            Encoding::Text => self.deserialize_text(),
            Encoding::Gron => self.deserialize_gron(),
            Encoding::HCL => self.deserialize_hcl(),
        }
    }

    fn deserialize_yaml(&mut self) -> Result<Value> {
        let values = serde_yaml::Deserializer::from_reader(&mut self.reader)
            .map(Value::deserialize)
            .collect::<Result<Vec<_>, _>>()?;

        // If this was not multi-document YAML, just take the first document's value without
        // wrapping it into an array.
        if values.len() == 1 {
            Ok(values[0].clone())
        } else {
            Ok(Value::Array(values))
        }
    }

    fn deserialize_json(&mut self) -> Result<Value> {
        Ok(serde_json::from_reader(&mut self.reader)?)
    }

    fn deserialize_toml(&mut self) -> Result<Value> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        Ok(toml::de::from_slice(&buf)?)
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
                    let headers: Vec<Value> = headers?;

                    Value::Array(
                        iter.map(|record| {
                            Ok(headers
                                .iter()
                                .zip(record?.into_iter())
                                .map(|(k, v)| (k.to_string_unquoted(), v))
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
                .filter(|m| !m.is_empty())
                .map(|m| Ok(serde_json::to_value(m)?))
                .collect::<Result<_>>()?,
        ))
    }

    fn deserialize_gron(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;

        let map = GronStatements::parse(&s)?
            .iter()
            .map(|statement| {
                Ok((
                    statement.path().to_owned(),
                    serde_json::from_str(statement.value())?,
                ))
            })
            .collect::<Result<Map<_, _>>>()?;

        Ok(Value::Object(map))
    }

    fn deserialize_hcl(&mut self) -> Result<Value> {
        Ok(hcl::from_reader(&mut self.reader)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_deserialize_yaml() {
        let mut de = Deserializer::new("---\nfoo: bar".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::YAML).unwrap(),
            json!({"foo": "bar"})
        );
    }

    #[test]
    fn test_deserialize_yaml_multi() {
        let mut de = Deserializer::new("---\nfoo: bar\n---\nbaz: qux".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::YAML).unwrap(),
            json!([{"foo": "bar"}, {"baz": "qux"}])
        );
    }

    #[test]
    fn test_deserialize_csv() {
        let mut de = Deserializer::new("header1,header2\ncol1,col2".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::CSV).unwrap(),
            json!([["col1", "col2"]])
        );

        let mut de = DeserializerBuilder::new()
            .csv_without_headers(true)
            .build("row1col1,row1col2\nrow2col1,row2col2".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::CSV).unwrap(),
            json!([["row1col1", "row1col2"], ["row2col1", "row2col2"]])
        );

        let mut de = DeserializerBuilder::new()
            .csv_headers_as_keys(true)
            .build("header1,header2\nrow1col1,row1col2\nrow2col1,row2col2".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::CSV).unwrap(),
            json!([{"header1":"row1col1", "header2":"row1col2"}, {"header1":"row2col1", "header2":"row2col2"}])
        );

        let mut de = DeserializerBuilder::new()
            .csv_delimiter(b'|')
            .build("header1|header2\ncol1|col2".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::CSV).unwrap(),
            json!([["col1", "col2"]])
        );
    }

    #[test]
    fn test_deserialize_text() {
        let mut de = Deserializer::new("one\ntwo\nthree\n".as_bytes());
        assert_eq!(
            de.deserialize(Encoding::Text).unwrap(),
            json!(["one", "two", "three"])
        );

        let buf: &[u8] = &[];
        let mut de = Deserializer::new(buf);
        assert_eq!(de.deserialize(Encoding::Text).unwrap(), json!([]));
    }
}
