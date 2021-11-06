//! This module provides a `Deserializer` which supports deserializing input data with various
//! encodings into a `Value`.

use crate::{Encoding, Result, Value};
use serde::Deserialize;

/// Options for the `Deserializer`. The options are context specific and may only be honored when
/// deserializing from a certain `Encoding`.
#[derive(Debug, Default, Clone, PartialEq)]
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
/// let deserializer = DeserializerBuilder::new()
///     .csv_delimiter(b'\t')
///     .build(Encoding::Csv);
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

    /// Builds the `Deserializer` for the given `Encoding`.
    pub fn build(&self, encoding: Encoding) -> Deserializer {
        Deserializer::new(encoding, self.opts.clone())
    }
}

/// A `Deserializer` can deserialize input data from a reader into a `Value`.
pub struct Deserializer {
    encoding: Encoding,
    opts: DeserializeOptions,
}

impl Deserializer {
    /// Creates a new `Deserializer` for `Encoding` with options.
    pub fn new(encoding: Encoding, opts: DeserializeOptions) -> Self {
        Self { encoding, opts }
    }

    /// Reads input data from the given reader and deserializes it in a `Value`.
    ///
    /// ## Example
    ///
    /// ```
    /// use dts::{de::DeserializerBuilder, Encoding};
    /// use serde_json::json;
    ///
    /// let de = DeserializerBuilder::new().build(Encoding::Json);
    ///
    /// let mut buf = r#"["foo"]"#.as_bytes();
    /// let value = de.deserialize(&mut buf).unwrap();
    ///
    /// assert_eq!(value, json!(["foo"]));
    /// ```
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
            Encoding::Csv => deserialize_csv(reader, &self.opts),
            Encoding::Pickle => deserialize_pickle(reader),
            Encoding::QueryString => deserialize_query_string(reader),
            Encoding::Xml => deserialize_xml(reader),
            Encoding::Text => deserialize_text(reader),
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

fn deserialize_csv<R>(reader: &mut R, opts: &DeserializeOptions) -> Result<Value>
where
    R: std::io::Read,
{
    let keep_first_line = opts.csv_without_headers || opts.csv_headers_as_keys;

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(!keep_first_line)
        .delimiter(opts.csv_delimiter.unwrap_or(b','))
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
                .collect::<Result<_>>()?,
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

fn deserialize_xml<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    Ok(serde_xml_rs::from_reader(reader)?)
}

fn deserialize_text<R>(reader: &mut R) -> Result<Value>
where
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    Ok(Value::Array(
        s.lines()
            .map(|line| Ok(serde_json::to_value(line)?))
            .collect::<Result<_>>()?,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_deserialize_yaml() {
        let de = DeserializerBuilder::new().build(Encoding::Yaml);

        let mut buf = "---\nfoo: bar".as_bytes();

        assert_eq!(de.deserialize(&mut buf).unwrap(), json!({"foo": "bar"}));
    }

    #[test]
    fn test_deserialize_yaml_multi() {
        let de = DeserializerBuilder::new().build(Encoding::Yaml);

        let mut buf = "---\nfoo: bar\n---\nbaz: qux".as_bytes();

        assert_eq!(de.deserialize(&mut buf).unwrap(), json!({"foo": "bar"}));

        let de = DeserializerBuilder::new()
            .all_documents(true)
            .build(Encoding::Yaml);

        let mut buf = "---\nfoo: bar\n---\nbaz: qux".as_bytes();

        assert_eq!(
            de.deserialize(&mut buf).unwrap(),
            json!([{"foo": "bar"}, {"baz": "qux"}])
        );
    }

    #[test]
    fn test_deserialize_csv() {
        let de = DeserializerBuilder::new().build(Encoding::Csv);

        let mut buf = "header1,header2\ncol1,col2".as_bytes();

        assert_eq!(de.deserialize(&mut buf).unwrap(), json!([["col1", "col2"]]));

        let de = DeserializerBuilder::new()
            .csv_without_headers(true)
            .build(Encoding::Csv);

        let mut buf = "row1col1,row1col2\nrow2col1,row2col2".as_bytes();

        assert_eq!(
            de.deserialize(&mut buf).unwrap(),
            json!([["row1col1", "row1col2"], ["row2col1", "row2col2"]])
        );

        let de = DeserializerBuilder::new()
            .csv_headers_as_keys(true)
            .build(Encoding::Csv);

        let mut buf = "header1,header2\nrow1col1,row1col2\nrow2col1,row2col2".as_bytes();

        assert_eq!(
            de.deserialize(&mut buf).unwrap(),
            json!([{"header1":"row1col1", "header2":"row1col2"}, {"header1":"row2col1", "header2":"row2col2"}])
        );

        let de = DeserializerBuilder::new()
            .csv_delimiter(b'|')
            .build(Encoding::Csv);

        let mut buf = "header1|header2\ncol1|col2".as_bytes();

        assert_eq!(de.deserialize(&mut buf).unwrap(), json!([["col1", "col2"]]));
    }

    #[test]
    fn test_deserialize_text() {
        let de = DeserializerBuilder::new().build(Encoding::Text);

        let mut buf = "one\ntwo\nthree\n".as_bytes();

        assert_eq!(
            de.deserialize(&mut buf).unwrap(),
            json!(["one", "two", "three"])
        );

        let mut buf: &[u8] = &[];

        assert_eq!(de.deserialize(&mut buf).unwrap(), json!([]));
    }
}
