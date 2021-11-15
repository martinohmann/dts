//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

use dts::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    de::Deserializer,
    detect_encoding,
    io::{Reader, Writer},
    ser::Serializer,
    transform, Encoding, Value,
};

fn deserialize(file: &Path, opts: &InputOptions) -> Result<Value> {
    let encoding = detect_encoding(opts.input_encoding, file)
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let reader = Reader::new(file)
        .with_context(|| format!("failed to open input file: {}", file.display()))?;
    let mut de = Deserializer::with_options(reader, opts.into());

    de.deserialize(encoding)
        .context(format!("failed to deserialize {}", encoding))
}

fn transform(value: &Value, opts: &TransformOptions) -> Result<Value> {
    transform::apply_chain(&opts.transform, value).context("failed to transform value")
}

fn serialize(value: &Value, opts: &OutputOptions) -> Result<()> {
    // Output file or stdout.
    let file = &opts
        .output_file
        .clone()
        .unwrap_or_else(|| PathBuf::from("-"));

    let encoding = detect_encoding(opts.output_encoding, file).unwrap_or(Encoding::Json);

    let writer = Writer::new(file)
        .with_context(|| format!("failed to open output file: {}", file.display()))?;
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
        1 => deserialize(&files[0], &opts.input)?,
        _ => Value::Array(
            files
                .iter()
                .map(|file| deserialize(file, &opts.input))
                .collect::<Result<Vec<_>>>()?,
        ),
    };

    let value = transform(&value, &opts.transform)?;

    serialize(&value, &opts.output)
}
