#![doc = include_str!("../../README.md")]
#![deny(missing_docs)]

mod args;

use anyhow::{anyhow, Context, Result};
use args::{InputOptions, Options, OutputOptions, TransformOptions};

use clap::{App, IntoApp, Parser};
use clap_generate::{generate, Shell};
use dts_core::{de::Deserializer, ser::Serializer};
use dts_core::{transform, Encoding, Error, Sink, Source, Value};
use rayon::prelude::*;
use std::io::{self, BufReader, BufWriter};

#[cfg(feature = "color")]
mod color;
#[cfg(feature = "color")]
use std::path::Path;

fn deserialize(source: &Source, opts: &InputOptions) -> Result<Value> {
    let encoding = opts
        .input_encoding
        .or_else(|| source.encoding())
        .context("Unable to detect input encoding, please provide it explicitly via -i")?;

    let reader = source
        .to_reader()
        .with_context(|| format!("Failed to create reader for source `{}`", source))?;

    let mut de = Deserializer::with_options(BufReader::new(reader), opts.into());

    de.deserialize(encoding)
        .with_context(|| format!("Failed to deserialize `{}` from `{}`", encoding, source))
}

fn deserialize_many(sources: &[Source], opts: &InputOptions) -> Result<Value> {
    let results = if opts.continue_on_error {
        sources
            .par_iter()
            .filter_map(|src| match deserialize(src, opts) {
                Ok(val) => Some((src, val)),
                Err(_) => {
                    eprintln!("Warning: Source `{}` skipped due to errors", src);
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        sources
            .par_iter()
            .map(|src| deserialize(src, opts).map(|val| (src, val)))
            .collect::<Result<Vec<_>>>()?
    };

    if opts.file_paths {
        Ok(Value::Object(
            results
                .into_iter()
                .map(|res| (res.0.to_string(), res.1))
                .collect(),
        ))
    } else {
        Ok(Value::Array(results.into_iter().map(|res| res.1).collect()))
    }
}

fn transform(value: Value, opts: &TransformOptions) -> Result<Value> {
    transform::apply_chain(&opts.transform, value).context("Failed to transform value")
}

fn serialize_writer<W>(
    writer: W,
    encoding: Encoding,
    value: &Value,
    opts: &OutputOptions,
) -> Result<()>
where
    W: io::Write,
{
    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    match ser.serialize(encoding, value) {
        Ok(()) => Ok(()),
        Err(Error::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(feature = "color")]
fn serialize_colored(encoding: Encoding, value: &Value, opts: &OutputOptions) -> Result<()> {
    let mut buf = Vec::with_capacity(256);

    serialize_writer(&mut buf, encoding, value, opts)?;

    // Pseudo filename which will determine the syntax highlighting used by the PrettyPrinter.
    let filename = Path::new("out").with_extension(encoding.as_str());

    // The PrettyPrinter will always write to io::Stdout.
    let mut printer = color::PrettyPrinter::new();

    printer
        .input(color::Input::from_bytes(&buf).name(filename))
        .theme(opts.theme.as_deref().unwrap_or("base16"))
        .print()
        .map(|_| ())
        .map_err(|err| anyhow!("{}", err))
}

#[cfg(feature = "color")]
fn serialize(sink: &Sink, value: &Value, opts: &OutputOptions) -> Result<()> {
    let encoding = opts
        .output_encoding
        .or_else(|| sink.encoding())
        .unwrap_or(Encoding::Json);

    let res = if sink == &Sink::Stdout && opts.color.should_colorize() {
        // Slow colorful path.
        serialize_colored(encoding, value, opts)
    } else {
        // Fast path.
        let writer = sink
            .to_writer()
            .with_context(|| format!("Failed to create writer for sink `{}`", sink))?;

        serialize_writer(writer, encoding, value, opts)
    };

    res.with_context(|| format!("Failed to serialize `{}` to `{}`", encoding, sink))
}

#[cfg(not(feature = "color"))]
fn serialize(sink: &Sink, value: &Value, opts: &OutputOptions) -> Result<()> {
    let encoding = opts
        .output_encoding
        .or_else(|| sink.encoding())
        .unwrap_or(Encoding::Json);

    let writer = sink
        .to_writer()
        .with_context(|| format!("Failed to create writer for sink `{}`", sink))?;

    serialize_writer(writer, encoding, value, opts)
        .with_context(|| format!("Failed to serialize `{}` to `{}`", encoding, sink))
}

fn serialize_many(sinks: &[Sink], value: &mut Value, opts: &OutputOptions) -> Result<()> {
    let values = match value.as_array_mut() {
        Some(values) => {
            if sinks.len() < values.len() {
                // There are more values than files. The last file takes an array of the left
                // over values.
                let rest = values.split_off(sinks.len() - 1);
                values.push(Value::Array(rest));
            }

            values
        }
        None => {
            return Err(anyhow!(
                "When using multiple output files, the data must be an array"
            ))
        }
    };

    if sinks.len() > values.len() {
        eprintln!(
            "Warning: Skipping {} output files due to lack of data",
            sinks.len() - values.len()
        );
    }

    sinks
        .iter()
        .zip(values.iter())
        .try_for_each(|(file, value)| serialize(file, value, opts))
}

fn print_completions(app: &mut App, shell: Shell) {
    generate(shell, app, app.get_name().to_string(), &mut io::stdout());
}

fn main() -> Result<()> {
    let opts = Options::parse();

    if let Some(shell) = opts.generate_completion {
        let mut app = Options::into_app();
        print_completions(&mut app, shell);
        std::process::exit(0);
    }

    #[cfg(feature = "color")]
    if opts.output.list_themes {
        let assets = color::HighlightingAssets::from_binary();
        assets.themes().for_each(|theme| println!("{}", theme));
        std::process::exit(0);
    }

    let mut sources = Vec::with_capacity(opts.sources.len());

    // If sources contains directories, force deserialization into a collection (array or object
    // with sources as keys depending on the input options) even if all directory globs only
    // produce a zero or one sources. This will ensure that deserializing the files that resulted
    // from directory globs always produces a consistent structure of the data.
    let dir_sources = opts.sources.iter().any(|s| s.is_dir());

    for source in opts.sources {
        match source.as_path() {
            Some(path) => {
                if path.is_dir() {
                    let pattern = opts
                        .input
                        .glob
                        .as_ref()
                        .context("--glob is required if sources contain directories")?;

                    let mut matches = source.glob_files(pattern)?;

                    sources.append(&mut matches);
                } else {
                    sources.push(path.into());
                }
            }
            None => sources.push(source),
        }
    }

    if sources.is_empty() && !atty::is(atty::Stream::Stdin) {
        // Input is piped on stdin.
        sources.push(Source::Stdin);
    }

    let value = match (sources.len(), dir_sources) {
        (0, false) => return Err(anyhow!("Input file or data on stdin expected")),
        (1, false) => deserialize(&sources[0], &opts.input)?,
        (_, _) => deserialize_many(&sources, &opts.input)?,
    };

    let mut value = transform(value, &opts.transform)?;

    let sinks = opts.sinks;

    if sinks.len() <= 1 {
        serialize(sinks.get(0).unwrap_or(&Sink::Stdout), &value, &opts.output)
    } else {
        serialize_many(&sinks, &mut value, &opts.output)
    }
}
