//! Command line arguments for dts.

use crate::{de::DeserializeOptions, ser::SerializeOptions};
use crate::{transform::Transformation, Encoding, Error, Result, Sink, Source};
use clap::{ArgSettings, Args, Parser, ValueHint};
use regex::Regex;
use std::str::FromStr;
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
    /// Input sources.
    ///
    /// If multiple files are provides, the decoded data is read into an array. The input files
    /// many also be remote URLs. Data may also be provided on stdin. If stdin is used in
    /// combination with one or more input files, the data from stdin will be read into the first
    /// element of the resulting array.
    #[clap(name = "SOURCE")]
    pub sources: Vec<Source>,

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

    /// Number of threads to use for deserialization.
    #[clap(short = 'j', long, default_value = "10")]
    pub threads: usize,

    /// Glob pattern for directories.
    ///
    /// Required if any of the input paths is a directory. Ignored otherwise.
    #[clap(long)]
    pub glob: Option<String>,

    /// Read input into a map keyed by file path of the origin file.
    ///
    /// If multiple input files or at least one directory is provided, this reads the result into
    /// a map keyed by file path instead of an array. If only one input file is provided, this
    /// option is ignored.
    #[clap(short = 'P', long)]
    pub file_paths: bool,
}

impl From<&InputOptions> for DeserializeOptions {
    fn from(opts: &InputOptions) -> Self {
        Self {
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
    /// Comma-separated list of transformation options. Can be specified multiple times.
    ///
    /// Transformation options have a short and a long form and optionally take a value separated
    /// by `=`. For some options the value is mandatory. Transformations are applied in the order
    /// they are defined.
    ///
    /// ## Example
    ///
    /// dts input.json --transform f,F,jsonpath='$.items' -t remove-empty-values
    ///
    /// The following transform options are available:
    ///
    /// ## JSONPath query filter
    ///
    /// Option: `j=<query>` or `jsonpath=<query>`.
    ///
    /// Select data from the decoded input via jsonpath query. Can be specified multiple times to
    /// allow starting the filtering from the root element again.
    ///
    /// See <https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators> for supported
    /// operators.
    ///
    /// When using a jsonpath query, the result will always be shaped like an array with zero or
    /// more elements. See --flatten-arrays if you want to remove one level of nesting on single
    /// element filter results.
    ///
    /// ## Flatten arrays
    ///
    /// Option: `f` or `flatten-arrays`.
    ///
    /// Remove one level of nesting if the data is shaped like an array. Can be specified multiple
    /// times.
    ///
    /// If the has only one element the array will be removed
    /// entirely, leaving the single element as output.
    ///
    /// This is applied as the last transformation before serializing into the output encoding. Can
    /// be used in combination with --jsonpath to flatten single element filter results.
    ///
    /// ## Flatten keys
    ///
    /// Option: `F[=<prefix>]` or `flatten-keys[=<prefix>]`.
    ///
    /// Flattens the input to an object with flat keys.
    ///
    /// The flag accepts an optional value for the key prefix. If the value is omitted, the key
    /// prefix is "data".
    ///
    /// The structure of the result is similar to the output of `gron`:
    /// <https://github.com/TomNomNom/gron>.
    ///
    /// ## Remove empty values
    ///
    /// Option: `r` or `remove-empty-values`.
    ///
    /// Recursively removes nulls, empty arrays and empty objects from the data.
    ///
    /// Top level empty values are not removed.
    ///
    /// ## Deep merge
    ///
    /// Option: `m` or `deep-merge`.
    ///
    /// If the data is an array, all children are merged into one from left to right. Otherwise
    /// this is a no-op.
    ///
    /// Arrays are merged by collecting the values of all children.
    ///
    /// Objects are merged by creating a new object with all keys from the left and right value.
    /// Keys present on sides are merged recursively.
    ///
    /// In all other cases, the rightmost value is taken.
    #[clap(short = 't', long, parse(try_from_str = Transformation::from_str))]
    #[clap(multiple_occurrences = true, number_of_values = 1)]
    pub transform: Vec<Transformation>,
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

    /// Output file. If absent, the encoded data is written to stdout.
    #[clap(short = 'O', long, value_hint = ValueHint::FilePath)]
    #[clap(default_value = "-", setting = ArgSettings::HideDefaultValue)]
    pub output_file: Sink,

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
