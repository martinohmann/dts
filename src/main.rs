//! trnscd is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, ValueHint};
use jsonpath_rust::JsonPathQuery;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use trnscd::{
    de::DeserializerBuilder, detect_encoding, ser::SerializerBuilder, Encoding, Reader, Value,
    Writer,
};

/// Simple tool to transcode between different encodings.
#[derive(Parser, Debug)]
#[clap(name = "trnscd", version)]
struct Options {
    /// If stdin is not a pipe, the first file is read from. Otherwise it is treated as the output
    /// file. It is possible to provide multiple output files if the data resembles an array. Each
    /// output file will receive an array element. The last output file collects the remaining
    /// elements if there are more elements than files. Passing '-' as filename or providing no
    /// output files will write the data to stdout instead
    #[clap(name = "FILE", parse(from_os_str), value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Options for deserializing the input.
    #[clap(flatten)]
    input: InputOptions,

    /// Options for serializing the output.
    #[clap(flatten)]
    output: OutputOptions,

    /// Options for data transformations performed after deserializing from the input encoding but
    /// before serializing to the output encoding.
    #[clap(flatten)]
    transform: TransformOptions,
}

#[derive(Args, Debug)]
struct InputOptions {
    /// Input encoding, if absent encoding will be detected from input file extension
    #[clap(arg_enum, short = 'i', long)]
    input_encoding: Option<Encoding>,

    /// Deserialize inputs that can contain multiple documents (e.g. YAML) into an array.
    /// Otherwise, only deserialize the first document
    #[clap(short = 'A', long)]
    all_documents: bool,

    /// If this flag is absent, the first line of CSV or TSV input is treated as headers and will
    /// be discarded
    #[clap(long)]
    csv_without_headers: bool,

    /// When reading CSV or TSV, this flag will deserialize the input into an array of maps with
    /// each field keyed by the corresponding header value. Otherwise, the input is deserialized
    /// into an array of arrays
    #[clap(long)]
    csv_headers_as_keys: bool,
}

#[derive(Args, Debug)]
struct OutputOptions {
    /// Output encoding, if absent encoding will be detected from output file extension
    #[clap(arg_enum, short = 'o', long)]
    output_encoding: Option<Encoding>,

    /// Produce pretty output if supported by the encoder
    #[clap(short = 'p', long)]
    pretty: bool,

    /// Add a trailing newline to the output
    #[clap(short = 'n', long)]
    newline: bool,

    /// When the input is an array of objects and the output encoding is CSV or TSV, the field
    /// names of the first object will be used as CSV headers. Field values of all following
    /// objects will be matched to the right CSV column based on their key. Missing fields produce
    /// empty columns while excess fields are ignored.
    #[clap(long)]
    keys_as_csv_headers: bool,
}

#[derive(Args, Debug)]
struct TransformOptions {
    /// Select data from the decoded input via jsonpath query before serializing it into the output
    /// encoding. See https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators for
    /// supported operators. Can be specified multiple times to allow filtering intermediate
    /// results from the root again.
    ///
    /// When using a jsonpath query, the result will always be shaped like an array with zero or
    /// more elements. See --flatten if you want to remove one level of nesting on single element
    /// filter results.
    #[clap(short = 'j', long, multiple_occurrences = true)]
    jsonpath: Vec<String>,

    /// If the data is shaped like an array and has only one element, flatten it to the element by
    /// removing one level of nesting. This is applied as the last transformation before
    /// serializing into the output encoding.
    ///
    /// Can be used in combination with --jsonpath to flatten single element filter results.
    #[clap(long)]
    flatten: bool,
}

fn deserialize<P>(file: Option<P>, opts: &InputOptions) -> Result<Value>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.input_encoding, file.as_ref())
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let de = DeserializerBuilder::new()
        .all_documents(opts.all_documents)
        .csv_without_headers(opts.csv_without_headers)
        .csv_headers_as_keys(opts.csv_headers_as_keys)
        .build(encoding);

    let mut reader = Reader::new(file).context("failed to open input file")?;

    de.deserialize(&mut reader)
        .context(format!("failed to deserialize {}", encoding))
}

fn transform(mut value: Value, opts: &TransformOptions) -> Result<Value> {
    for selector in &opts.jsonpath {
        value = value
            .path(selector)
            .map_err(|e| anyhow!(e))
            .context("invalid jsonpath query")?;
    }

    if let Some(array) = value.as_array() {
        if opts.flatten && array.len() == 1 {
            return Ok(array[0].clone());
        }
    }

    Ok(value)
}

fn serialize<P>(file: Option<P>, value: &Value, opts: &OutputOptions) -> Result<()>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.output_encoding, file.as_ref())
        .context("unable to detect output encoding, please provide it explicitly via -o")?;

    let ser = SerializerBuilder::new()
        .pretty(opts.pretty)
        .newline(opts.newline)
        .keys_as_csv_headers(opts.keys_as_csv_headers)
        .build(encoding);

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

    let value = deserialize(input_file, &opts.input)?;

    let mut value = transform(value, &opts.transform)?;

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
