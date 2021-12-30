#![doc = include_str!("../../README.md")]
#![deny(missing_docs)]

mod args;
#[cfg(feature = "color")]
mod color;
mod no_color;
mod paging;
mod utils;

use anyhow::{anyhow, Context, Result};
use args::{InputOptions, Options, OutputOptions, TransformOptions};
use clap::{App, IntoApp, Parser};
use clap_generate::{generate, Shell};
#[cfg(feature = "color")]
use color::{print_themes, ColoredStdoutWriter, HighlightingConfig};
use dts_core::{de::Deserializer, ser::Serializer};
use dts_core::{transform, Encoding, Error, Sink, Source};
use dts_json::Value;
use no_color::StdoutWriter;
use paging::PagingConfig;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

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

fn serialize(sink: &Sink, value: Value, opts: &OutputOptions) -> Result<()> {
    let encoding = opts
        .output_encoding
        .or_else(|| sink.encoding())
        .unwrap_or(Encoding::Json);

    let paging_config = PagingConfig::new(opts.paging, opts.pager.as_deref());

    let writer: Box<dyn io::Write> = match sink {
        #[cfg(feature = "color")]
        Sink::Stdout => {
            if opts.color.should_colorize() {
                let config = HighlightingConfig::new(paging_config, opts.theme.as_deref());
                Box::new(ColoredStdoutWriter::new(encoding, config))
            } else {
                Box::new(StdoutWriter::new(paging_config))
            }
        }
        #[cfg(not(feature = "color"))]
        Sink::Stdout => Box::new(StdoutWriter::new(paging_config)),
        Sink::Path(path) => Box::new(
            File::create(path)
                .with_context(|| format!("Failed to create writer for sink `{}`", sink))?,
        ),
    };

    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    match ser.serialize(encoding, value) {
        Ok(()) => Ok(()),
        Err(Error::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err),
    }
    .with_context(|| format!("Failed to serialize `{}` to `{}`", encoding, sink))
}

fn serialize_many(sinks: &[Sink], value: Value, opts: &OutputOptions) -> Result<()> {
    let values = match value {
        Value::Array(mut values) => {
            if sinks.len() < values.len() {
                // There are more values than files. The last file takes an array of the left
                // over values.
                let rest = values.split_off(sinks.len() - 1);
                values.push(Value::Array(rest));
            }

            values
        }
        _ => {
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
        .zip(values.into_iter())
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
        print_themes(opts.output.color)?;
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

    let value = transform(value, &opts.transform)?;

    let sinks = opts.sinks;

    if sinks.len() <= 1 {
        serialize(sinks.get(0).unwrap_or(&Sink::Stdout), value, &opts.output)
    } else {
        serialize_many(&sinks, value, &opts.output)
    }
}
