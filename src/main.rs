//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::{ArgSettings, Args, Parser, ValueHint};
use jsonpath_rust::JsonPathQuery;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use dts::{
    de::{DeserializeOptions, Deserializer},
    detect_encoding,
    ser::{SerializeOptions, Serializer},
    Encoding, Reader, Value, Writer,
};

/// Simple tool to transcode between different encodings.
///
/// The tool first deserializes data from the input data into an internal representation which
/// resembles JSON. As an optional step certain transformations can be applied to the data before
/// serializing into the output encoding.
///
/// Refer to the input, transform and output options below.
#[derive(Parser, Debug)]
#[clap(name = "dts", version)]
struct Options {
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
    files: Vec<PathBuf>,

    /// Options for deserializing the input.
    #[clap(flatten)]
    input: InputOptions,

    /// Options for data transformations performed after deserializing from the input encoding but
    /// before serializing to the output encoding.
    #[clap(flatten)]
    transform: TransformOptions,

    /// Options for serializing the output.
    #[clap(flatten)]
    output: OutputOptions,
}

#[derive(Args, Debug)]
#[clap(help_heading = "INPUT OPTIONS")]
struct InputOptions {
    /// Set the input encoding. If absent encoding will be detected from input file extension.
    #[clap(arg_enum, short = 'i', long, setting = ArgSettings::HidePossibleValues)]
    input_encoding: Option<Encoding>,

    /// Deserialize inputs that can contain multiple documents (e.g. YAML) into an array.
    ///
    /// Otherwise, only deserialize the first document.
    #[clap(short = 'A', long)]
    all_documents: bool,

    /// Indicate that CSV input does not include a header row.
    ///
    /// If this flag is absent, the first line of CSV input is treated as headers and will be
    /// discarded.
    #[clap(long)]
    csv_without_headers: bool,

    /// Use CSV headers as keys for the row columns.
    ///
    /// When reading CSV, this flag will deserialize the input into an array of maps with each
    /// field keyed by the corresponding header value. Otherwise, the input is deserialized into an
    /// array of arrays.
    #[clap(long)]
    csv_headers_as_keys: bool,
}

impl From<&InputOptions> for DeserializeOptions {
    fn from(opts: &InputOptions) -> Self {
        Self {
            all_documents: opts.all_documents,
            csv_headers_as_keys: opts.csv_headers_as_keys,
            csv_without_headers: opts.csv_without_headers,
        }
    }
}

#[derive(Args, Debug)]
#[clap(help_heading = "TRANSFORM OPTIONS")]
struct TransformOptions {
    /// Select data from the decoded input via jsonpath query. Can be specified multiple times to
    /// allow starting the filtering from the root element again.
    ///
    /// See https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators for supported
    /// operators.
    ///
    /// When using a jsonpath query, the result will always be shaped like an array with zero or
    /// more elements. See --flatten if you want to remove one level of nesting on single element
    /// filter results.
    #[clap(short = 'j', long, multiple_occurrences = true, number_of_values = 1)]
    jsonpath: Vec<String>,

    /// Remove one level of nesting if the data is shaped like an array. Can be specified multiple
    /// times.
    ///
    /// If the has only one element the array will be removed
    /// entirely, leaving the single element as output.
    ///
    /// This is applied as the last transformation before serializing into the output encoding. Can
    /// be used in combination with --jsonpath to flatten single element filter results.
    #[clap(short, long, parse(from_occurrences))]
    flatten: u8,
}

#[derive(Args, Debug)]
#[clap(help_heading = "OUTPUT OPTIONS")]
struct OutputOptions {
    /// Set the output encoding. If absent encoding will be detected from output file extension.
    #[clap(arg_enum, short = 'o', long, setting = ArgSettings::HidePossibleValues)]
    output_encoding: Option<Encoding>,

    /// Produce pretty output if supported by the encoder.
    #[clap(short = 'p', long)]
    pretty: bool,

    /// Add a trailing newline to the output.
    #[clap(short = 'n', long)]
    newline: bool,

    /// Use object keys of the first item as CSV headers.
    ///
    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will be
    /// matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    #[clap(long)]
    keys_as_csv_headers: bool,
}

impl From<&OutputOptions> for SerializeOptions {
    fn from(opts: &OutputOptions) -> Self {
        Self {
            pretty: opts.pretty,
            newline: opts.newline,
            keys_as_csv_headers: opts.keys_as_csv_headers,
        }
    }
}

fn deserialize<P>(file: Option<P>, opts: &InputOptions) -> Result<Value>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.input_encoding, file.as_ref())
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let de = Deserializer::new(encoding, opts.into());

    let mut reader = Reader::new(file).context("failed to open input file")?;

    de.deserialize(&mut reader)
        .context(format!("failed to deserialize {}", encoding))
}

fn transform(value: &mut Value, opts: &TransformOptions) -> Result<()> {
    for selector in &opts.jsonpath {
        *value = value
            .clone()
            .path(selector)
            .map_err(|e| anyhow!(e))
            .context("invalid jsonpath query")?;
    }

    for _ in 0..opts.flatten {
        flatten(value)
    }

    Ok(())
}

fn flatten(value: &mut Value) {
    if let Some(array) = value.as_array() {
        if array.len() == 1 {
            *value = array[0].clone();
        } else {
            *value = Value::Array(
                array
                    .iter()
                    .map(|v| match v {
                        Value::Array(a) => a.clone(),
                        _ => vec![v.clone()],
                    })
                    .flatten()
                    .collect(),
            )
        }
    }
}

fn serialize<P>(file: Option<P>, value: &Value, opts: &OutputOptions) -> Result<()>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.output_encoding, file.as_ref())
        .context("unable to detect output encoding, please provide it explicitly via -o")?;

    let ser = Serializer::new(encoding, opts.into());

    let mut writer = Writer::new(file).context("failed to open output file")?;

    ser.serialize(&mut writer, value)
        .context(format!("failed to serialize {}", encoding))
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut files = VecDeque::from(opts.files.clone());

    // If stdin is not a pipe, use the first filename as the input and remove it from the list.
    // Otherwise it's an output filename.
    let input_file = match atty::is(atty::Stream::Stdin) {
        true => Some(
            files
                .pop_front()
                .ok_or_else(|| anyhow!("input file or data on stdin expected"))?,
        ),
        false => None,
    };

    let mut value = deserialize(input_file, &opts.input)?;

    transform(&mut value, &opts.transform)?;

    if files.len() <= 1 {
        serialize(files.get(0), &value, &opts.output)
    } else {
        let values = match value.as_array_mut() {
            Some(values) => {
                if files.len() < values.len() {
                    // There are more values than files. The last file takes an array of the left
                    // over values.
                    let rest = values.split_off(files.len() - 1);
                    values.push(Value::Array(rest));
                }

                values
            }
            None => {
                return Err(anyhow!(
                    "when using multiple output files, the data must be an array"
                ))
            }
        };

        files
            .iter()
            .zip(values.iter())
            .try_for_each(|(file, value)| serialize(Some(file), value, &opts.output))
    }
}
