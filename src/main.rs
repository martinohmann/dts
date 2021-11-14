//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
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

fn serialize(value: &Value, opts: &OutputOptions) -> Result<()> {
    let file = opts.output_file.as_ref();
    let encoding = detect_encoding(opts.output_encoding, file).unwrap_or(Encoding::Json);

    let writer = Writer::new(file).context("failed to open output file")?;
    let mut ser = Serializer::with_options(writer, opts.into());

    ser.serialize(encoding, value)
        .context(format!("failed to serialize {}", encoding))
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut files = opts.files.clone();

    if !atty::is(atty::Stream::Stdin) {
        // Input is piped on stdin.
        files.insert(0, Path::new("-").to_path_buf());
    }

    let value = match files.len() {
        0 => return Err(anyhow!("input file or data on stdin expected")),
        1 => deserialize(files.get(0), &opts.input)?,
        _ => Value::Array(
            files
                .iter()
                .map(|file| deserialize(Some(file), &opts.input))
                .collect::<Result<Vec<_>>>()?,
        ),
    };

    let value = transform(&value, &opts.transform)?;

    serialize(&value, &opts.output)
}
