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
    transform, Value,
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

fn transform(value: &mut Value, opts: &TransformOptions) -> Result<()> {
    opts.jsonpath
        .iter()
        .try_for_each(|query| transform::filter_in_place(value, query))
        .context("invalid jsonpath query")?;

    (0..opts.flatten).for_each(|_| transform::flatten_in_place(value));
    Ok(())
}

fn serialize<P>(file: Option<P>, value: &Value, opts: &OutputOptions) -> Result<()>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.output_encoding, file.as_ref())
        .context("unable to detect output encoding, please provide it explicitly via -o")?;

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
