//! Command line arguments for dts.

use crate::{de::DeserializeOptions, ser::SerializeOptions, Encoding, Error, Result};
use clap::{ArgSettings, Args, Parser, ValueHint};
use regex::Regex;
use std::path::PathBuf;
use unescape::unescape;

/// Simple tool to transcode between different encodings.
///
/// The tool first deserializes data from the input data into an internal representation which
/// resembles JSON. As an optional step certain transformations can be applied to the data before
/// serializing into the output encoding.
///
/// Refer to the input, transform and output options below.
#[derive(Parser, Debug)]
#[clap(name = "dts", version)]
pub struct Options {
    /// Input and output files.
    ///
    /// If stdin is not a pipe, the first file is the input file that is read from. Otherwise it is
    /// treated as an output file.
    ///
    /// It is possible to provide multiple output files if the data resembles an array. Each output
    /// file will receive an array element. The last output file collects the remaining elements if
    /// there are more elements than files.
    ///
    /// Passing '-' as filename or providing no output files will write the data to stdout instead.
    #[clap(name = "FILE", parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,

    /// Options for deserializing the input.
    #[clap(flatten)]
    pub input: InputOptions,

    /// Options for data transformations performed after deserializing from the input encoding but
    /// before serializing to the output encoding.
    #[clap(flatten)]
    pub transform: TransformOptions,

    /// Options for serializing the output.
    #[clap(flatten)]
    pub output: OutputOptions,
}

/// Options that configure the behaviour of input deserialization.
#[derive(Args, Debug)]
#[clap(help_heading = "INPUT OPTIONS")]
pub struct InputOptions {
    /// Set the input encoding. If absent encoding will be detected from input file extension.
    #[clap(arg_enum, short = 'i', long, setting = ArgSettings::HidePossibleValues)]
    pub input_encoding: Option<Encoding>,

    /// Deserialize inputs that can contain multiple documents (e.g. YAML) into an array.
    ///
    /// Otherwise, only deserialize the first document.
    #[clap(short = 'A', long)]
    pub all_documents: bool,

    /// Indicate that CSV input does not include a header row.
    ///
    /// If this flag is absent, the first line of CSV input is treated as headers and will be
    /// discarded.
    #[clap(long)]
    pub csv_without_headers: bool,

    /// Use CSV headers as keys for the row columns.
    ///
    /// When reading CSV, this flag will deserialize the input into an array of maps with each
    /// field keyed by the corresponding header value. Otherwise, the input is deserialized into an
    /// array of arrays.
    #[clap(short = 'H', long)]
    pub csv_headers_as_keys: bool,

    /// Custom delimiter for CSV input.
    #[clap(short = 'd', long, parse(try_from_str = parse_csv_delimiter))]
    pub csv_input_delimiter: Option<u8>,

    /// Regex pattern to split text input at.
    #[clap(short = 's', long)]
    pub text_split_pattern: Option<Regex>,
}

impl From<&InputOptions> for DeserializeOptions {
    fn from(opts: &InputOptions) -> Self {
        Self {
            all_documents: opts.all_documents,
            csv_headers_as_keys: opts.csv_headers_as_keys,
            csv_without_headers: opts.csv_without_headers,
            csv_delimiter: opts.csv_input_delimiter,
            text_split_pattern: opts.text_split_pattern.clone(),
        }
    }
}

/// Options that configure the behaviour of data transformation.
#[derive(Args, Debug)]
#[clap(help_heading = "TRANSFORM OPTIONS")]
pub struct TransformOptions {
    /// Select data from the decoded input via jsonpath query. Can be specified multiple times to
    /// allow starting the filtering from the root element again.
    ///
    /// See <https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators> for supported
    /// operators.
    ///
    /// When using a jsonpath query, the result will always be shaped like an array with zero or
    /// more elements. See --flatten if you want to remove one level of nesting on single element
    /// filter results.
    #[clap(short = 'j', long, multiple_occurrences = true, number_of_values = 1)]
    pub jsonpath: Vec<String>,

    /// Remove one level of nesting if the data is shaped like an array. Can be specified multiple
    /// times.
    ///
    /// If the has only one element the array will be removed
    /// entirely, leaving the single element as output.
    ///
    /// This is applied as the last transformation before serializing into the output encoding. Can
    /// be used in combination with --jsonpath to flatten single element filter results.
    #[clap(short, long, parse(from_occurrences))]
    pub flatten: u8,
}

/// Options that configure the behaviour of output serialization.
#[derive(Args, Debug)]
#[clap(help_heading = "OUTPUT OPTIONS")]
pub struct OutputOptions {
    /// Set the output encoding. If absent encoding will be detected from output file extension.
    ///
    /// If the encoding is not explicitly set and it cannot be inferred from the output file
    /// extension (or the output is stdout), the fallback is to encode output as JSON.
    #[clap(arg_enum, short = 'o', long, setting = ArgSettings::HidePossibleValues)]
    pub output_encoding: Option<Encoding>,

    /// Produce pretty output if supported by the encoder.
    #[clap(short = 'p', long)]
    pub pretty: bool,

    /// Add a trailing newline to the output.
    #[clap(short = 'n', long)]
    pub newline: bool,

    /// Use object keys of the first item as CSV headers.
    ///
    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will be
    /// matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    #[clap(short = 'K', long)]
    pub keys_as_csv_headers: bool,

    /// Custom delimiter for CSV output.
    #[clap(short = 'D', long, parse(try_from_str = parse_csv_delimiter))]
    pub csv_output_delimiter: Option<u8>,

    /// Custom separator to join text output with.
    #[clap(short = 'J', long, parse(try_from_str = parse_unescaped))]
    pub text_join_separator: Option<String>,
}

impl From<&OutputOptions> for SerializeOptions {
    fn from(opts: &OutputOptions) -> Self {
        Self {
            pretty: opts.pretty,
            newline: opts.newline,
            keys_as_csv_headers: opts.keys_as_csv_headers,
            csv_delimiter: opts.csv_output_delimiter,
            text_join_separator: opts.text_join_separator.clone(),
        }
    }
}

fn parse_csv_delimiter(s: &str) -> Result<u8> {
    let unescaped = parse_unescaped(s)?;
    let bytes = unescaped.as_bytes();

    if bytes.len() == 1 {
        Ok(bytes[0])
    } else {
        Err(Error::new("expected single byte delimiter"))
    }
}

fn parse_unescaped(s: &str) -> Result<String> {
    unescape(s)
        .ok_or_else(|| Error::new(format!("string contains invalid escape sequences: '{}'", s)))
}
