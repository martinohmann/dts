//! This module provides a `Deserializer` which supports deserializing input data with various
//! encodings into a `Value`.

use crate::{Encoding, Result, Value};
use regex::Regex;
use serde::Deserialize;

/// Options for the `Deserializer`. The options are context specific and may only be honored when
/// deserializing from a certain `Encoding`.
#[derive(Debug, Default, Clone)]
pub struct DeserializeOptions {
    /// If the input is multi-document YAML, deserialize all documents into an array. Otherwise old
    /// deserialize the first document and discard the rest.
    pub all_documents: bool,
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

    /// If the input is multi-document YAML, deserialize all documents into an array. Otherwise old
    /// deserialize the first document and discard the rest.
    pub fn all_documents(&mut self, yes: bool) -> &mut Self {
        self.opts.all_documents = yes;
        self
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
        Deserializer::new(reader, self.opts.clone())
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
    /// Creates a new `Deserializer` for reader with options.
    pub fn new(reader: R, opts: DeserializeOptions) -> Self {
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
            Encoding::Ron => self.deserialize_ron(),
            Encoding::Toml => self.deserialize_toml(),
            Encoding::Json5 => self.deserialize_json5(),
            Encoding::Hjson => self.deserialize_hjson(),
            Encoding::Csv => self.deserialize_csv(),
            Encoding::Pickle => self.deserialize_pickle(),
            Encoding::QueryString => self.deserialize_query_string(),
            Encoding::Xml => self.deserialize_xml(),
            Encoding::Text => self.deserialize_text(),
        }
    }

    fn deserialize_yaml(&mut self) -> Result<Value> {
        let mut values = Vec::new();

        for doc in serde_yaml::Deserializer::from_reader(&mut self.reader) {
            let value = Value::deserialize(doc)?;

            if self.opts.all_documents {
                values.push(value);
            } else {
                return Ok(value);
            }
        }

        Ok(Value::Array(values))
    }

    fn deserialize_json(&mut self) -> Result<Value> {
        Ok(serde_json::from_reader(&mut self.reader)?)
    }

    fn deserialize_ron(&mut self) -> Result<Value> {
        Ok(ron::de::from_reader(&mut self.reader)?)
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

    fn deserialize_hjson(&mut self) -> Result<Value> {
        let mut s = String::new();
        self.reader.read_to_string(&mut s)?;
        Ok(deser_hjson::from_str(&s)?)
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
                    .collect::<Result<_>>()?,
            )
        };

        Ok(value)
    }

    fn deserialize_pickle(&mut self) -> Result<Value> {
        Ok(serde_pickle::from_reader(
            &mut self.reader,
            Default::default(),
        )?)
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
            None => Regex::new("\n")?,
        };

        Ok(Value::Array(
            pattern
                .split(&s)
                .filter(|m| !m.is_empty())
                .map(|m| Ok(serde_json::to_value(m)?))
                .collect::<Result<_>>()?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_deserialize_yaml() {
        let buf = "---\nfoo: bar".as_bytes();
        let mut de = DeserializerBuilder::new().build(buf);

        assert_eq!(
            de.deserialize(Encoding::Yaml).unwrap(),
            json!({"foo": "bar"})
        );
    }

    #[test]
    fn test_deserialize_yaml_multi() {
        let buf = "---\nfoo: bar\n---\nbaz: qux".as_bytes();
        let mut de = DeserializerBuilder::new().build(buf);

        assert_eq!(
            de.deserialize(Encoding::Yaml).unwrap(),
            json!({"foo": "bar"})
        );

        let buf = "---\nfoo: bar\n---\nbaz: qux".as_bytes();
        let mut de = DeserializerBuilder::new().all_documents(true).build(buf);

        assert_eq!(
            de.deserialize(Encoding::Yaml).unwrap(),
            json!([{"foo": "bar"}, {"baz": "qux"}])
        );
    }

    #[test]
    fn test_deserialize_csv() {
        let buf = "header1,header2\ncol1,col2".as_bytes();
        let mut de = DeserializerBuilder::new().build(buf);

        assert_eq!(
            de.deserialize(Encoding::Csv).unwrap(),
            json!([["col1", "col2"]])
        );

        let buf = "row1col1,row1col2\nrow2col1,row2col2".as_bytes();
        let mut de = DeserializerBuilder::new()
            .csv_without_headers(true)
            .build(buf);

        assert_eq!(
            de.deserialize(Encoding::Csv).unwrap(),
            json!([["row1col1", "row1col2"], ["row2col1", "row2col2"]])
        );

        let buf = "header1,header2\nrow1col1,row1col2\nrow2col1,row2col2".as_bytes();
        let mut de = DeserializerBuilder::new()
            .csv_headers_as_keys(true)
            .build(buf);

        assert_eq!(
            de.deserialize(Encoding::Csv).unwrap(),
            json!([{"header1":"row1col1", "header2":"row1col2"}, {"header1":"row2col1", "header2":"row2col2"}])
        );

        let buf = "header1|header2\ncol1|col2".as_bytes();
        let mut de = DeserializerBuilder::new().csv_delimiter(b'|').build(buf);

        assert_eq!(
            de.deserialize(Encoding::Csv).unwrap(),
            json!([["col1", "col2"]])
        );
    }

    #[test]
    fn test_deserialize_text() {
        let buf = "one\ntwo\nthree\n".as_bytes().to_vec();
        let mut de = DeserializerBuilder::new().build(buf);

        assert_eq!(
            de.deserialize(Encoding::Text).unwrap(),
            json!(["one", "two", "three"])
        );

        let buf: &[u8] = &[];
        let mut de = DeserializerBuilder::new().build(buf);

        assert_eq!(de.deserialize(Encoding::Text).unwrap(), json!([]));
    }
}
