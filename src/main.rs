//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::collections::VecDeque;
use std::path::Path;

use dts::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    de::Deserializer,
    detect_encoding,
    io::{Reader, Writer},
    ser::Serializer,
    transform, Encoding, Value,
};

fn deserialize<P>(file: Option<P>, opts: &InputOptions) -> Result<Value>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.input_encoding, file.as_ref())
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let reader = Reader::new(file).context("failed to open input file")?;
    let mut de = Deserializer::with_options(reader, opts.into());

    de.deserialize(encoding)
        .context(format!("failed to deserialize {}", encoding))
}

fn transform(value: &Value, opts: &TransformOptions) -> Result<Value> {
    let mut value = opts
        .jsonpath
        .iter()
        .try_fold(value.clone(), |value, query| {
            transform::filter_jsonpath(&value, query)
        })
        .context("invalid jsonpath query")?;

    for _ in 0..opts.flatten_arrays {
        value = transform::flatten_arrays(&value);
    }

    if let Some(prefix) = &opts.flatten_keys {
        value = transform::flatten_keys(&value, prefix)
    }

    if opts.remove_empty_values {
        value = transform::remove_empty_values(&value);
    }

    Ok(value)
}

fn serialize<P>(file: Option<P>, value: &Value, opts: &OutputOptions) -> Result<()>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.output_encoding, file.as_ref()).unwrap_or(Encoding::Json);

    let writer = Writer::new(file).context("failed to open output file")?;
    let mut ser = Serializer::with_options(writer, opts.into());

    ser.serialize(encoding, value)
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
    let mut value = transform(&value, &opts.transform)?;

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
