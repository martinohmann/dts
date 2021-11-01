use anyhow::{bail, Context, Result};
use clap::{Parser, ValueHint};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use trnscd::{
    de::{Deserializer, DeserializerBuilder},
    detect_encoding,
    ser::{Serializer, SerializerBuilder},
    Encoding, Reader, Value, Writer,
};

/// Simple tool to transcode between different encodings.
#[derive(Parser, Debug, Clone)]
#[clap(name = "trnscd")]
struct Options {
    /// Input encoding, if absent encoding will be detected from input file extension
    #[clap(arg_enum, short = 'i', long)]
    input_encoding: Option<Encoding>,

    /// Output encoding, if absent encoding will be detected from output file extension
    #[clap(arg_enum, short = 'o', long)]
    output_encoding: Option<Encoding>,

    /// Produce pretty output if supported by the encoder
    #[clap(short = 'p', long)]
    pretty: bool,

    /// Add a trailing newline to the output
    #[clap(short = 'n', long)]
    newline: bool,

    /// Deserialize inputs that can contain multiple documents (e.g. YAML) into an array.
    /// Otherwise, only deserialize the first document
    #[clap(short = 'A', long)]
    all_documents: bool,

    /// If this flag is absent, the first line of CSV or TSV input is treated as headers and will
    /// be discarded.
    #[clap(long)]
    csv_without_headers: bool,

    /// When reading CSV or TSV, this flag will deserialize the input into an array of maps with
    /// each field keyed by the corresponding header value. Otherwise, the input is deserialized
    /// into an array of arrays.
    #[clap(long)]
    csv_headers_as_keys: bool,

    /// If stdin is not a pipe, the first file is read from. Otherwise it is treated as the output
    /// file. It is possible to provide multiple output files if the data resembles an array. Each
    /// output file will receive an array element. The last output file collects the remaining
    /// elements if there are more elements than files. Passing '-' as filename or providing no
    /// output files will write the data to stdout instead.
    #[clap(name = "FILE", parse(from_os_str), value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,
}

fn build_deserializer<P>(opts: &Options, input: Option<P>) -> Result<Deserializer>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.input_encoding, input)
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    Ok(DeserializerBuilder::new()
        .all_documents(opts.all_documents)
        .csv_without_headers(opts.csv_without_headers)
        .csv_headers_as_keys(opts.csv_headers_as_keys)
        .build(encoding))
}

fn build_serializer<P>(opts: &Options, output: Option<P>) -> Result<Serializer>
where
    P: AsRef<Path>,
{
    let encoding = detect_encoding(opts.output_encoding, output)
        .context("unable to detect output encoding, please provide it explicitly via -o")?;

    Ok(SerializerBuilder::new()
        .pretty(opts.pretty)
        .newline(opts.newline)
        .build(encoding))
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut files = VecDeque::from(opts.files.clone());

    // If stdin is not a pipe, use the first filename as the input and remove it from the list.
    // Otherwise it's an output filename.
    let input = if atty::is(atty::Stream::Stdin) {
        files.pop_front()
    } else {
        None
    };

    let de = build_deserializer(&opts, input.as_ref())?;
    let ser = build_serializer(&opts, files.get(0))?;

    let mut reader = Reader::new(&input)?;

    let mut value = de.deserialize(&mut reader)?;

    if files.len() <= 1 {
        let mut writer = Writer::new(&files.get(0))?;
        ser.serialize(&mut writer, &value)
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
            None => bail!("when using multiple output files, the data must be an array"),
        };

        for (file, value) in files.iter().zip(values.iter()) {
            let mut writer = Writer::new(&Some(file))?;
            ser.serialize(&mut writer, value)?;
        }

        Ok(())
    }
}
