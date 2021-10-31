use anyhow::{Context, Result};
use clap::{Parser, ValueHint};
use std::path::PathBuf;

use trnscd::{
    de::{DeserializeOptions, Deserializer},
    detect_encoding,
    ser::{SerializeOptions, Serializer},
    Encoding, Reader, Writer,
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

    /// Indicates the first line of CSV or TSV input should not be treated as the headers.
    #[clap(long)]
    no_headers: bool,

    /// Input file, if absent or '-' input is read from stdin
    #[clap(name = "INPUT", parse(from_os_str), value_hint = ValueHint::FilePath)]
    input: Option<PathBuf>,

    /// Ouput file, if absent output is written to stdout
    #[clap(name = "OUTPUT", parse(from_os_str), value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,
}

impl Options {
    fn deserialize_opts(&self) -> DeserializeOptions {
        DeserializeOptions {
            all_documents: self.all_documents,
            no_headers: self.no_headers,
        }
    }

    fn deserializer(&self) -> Result<Deserializer> {
        let encoding = detect_encoding(self.input_encoding, self.input.as_ref())
            .context("unable to detect input encoding, please provide it explicitly via -i")?;

        Ok(Deserializer::new(encoding))
    }

    fn serialize_opts(&self) -> SerializeOptions {
        SerializeOptions {
            pretty: self.pretty,
            newline: self.newline,
        }
    }

    fn serializer(&self) -> Result<Serializer> {
        let encoding = detect_encoding(self.output_encoding, self.output.as_ref())
            .context("unable to detect output encoding, please provide it explicitly via -o")?;

        Ok(Serializer::new(encoding))
    }
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let de = opts.deserializer()?;
    let ser = opts.serializer()?;

    let reader = Reader::new(&opts.input)?;
    let value = de.deserialize(reader, opts.deserialize_opts())?;

    let writer = Writer::new(&opts.output)?;
    ser.serialize(writer, value, opts.serialize_opts())
}
