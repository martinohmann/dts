mod args;
#[cfg(feature = "color")]
mod highlighting;
mod output;
mod paging;
mod utils;

#[cfg(feature = "color")]
use crate::highlighting::{print_themes, ColoredStdoutWriter, SyntaxHighlighter};
use crate::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    output::StdoutWriter,
    paging::PagingConfig,
};
use anyhow::{anyhow, Context, Result};
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Shell};
use dts::{de::Deserializer, filter::Filter, ser::Serializer, Encoding, Error, Sink, Source};
use rayon::prelude::*;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufWriter, IsTerminal};

fn deserialize(source: &Source, opts: &InputOptions) -> Result<Value> {
    let reader = source
        .to_reader()
        .with_context(|| format!("failed to create reader for source `{}`", source))?;

    let encoding = opts
        .input_encoding
        .or_else(|| reader.encoding())
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let mut de = Deserializer::with_options(reader, opts.into());

    de.deserialize(encoding)
        .with_context(|| format!("failed to deserialize `{}` from `{}`", encoding, source))
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
    match &opts.jq_expression {
        Some(expr) => {
            let expr = match expr.strip_prefix('@') {
                Some(path) => fs::read_to_string(path)?,
                None => expr.to_owned(),
            };

            let filter = Filter::new(&expr)?;

            filter.apply(value).context("failed to transform value")
        }
        None => Ok(value),
    }
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
                let highlighter = SyntaxHighlighter::new(paging_config);
                let theme = opts.theme.as_deref();
                Box::new(ColoredStdoutWriter::new(highlighter, encoding, theme))
            } else {
                Box::new(StdoutWriter::new(paging_config))
            }
        }
        #[cfg(not(feature = "color"))]
        Sink::Stdout => Box::new(StdoutWriter::new(paging_config)),
        Sink::Path(path) => Box::new(
            File::create(path)
                .with_context(|| format!("failed to create writer for sink `{}`", sink))?,
        ),
    };

    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    match ser.serialize(encoding, value) {
        Ok(()) => Ok(()),
        Err(Error::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err),
    }
    .with_context(|| format!("failed to serialize `{}` to `{}`", encoding, sink))
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
                "when using multiple output files, the data must be an array"
            ))
        }
    };

    if sinks.len() > values.len() {
        eprintln!(
            "Warning: skipping {} output files due to lack of data",
            sinks.len() - values.len()
        );
    }

    sinks
        .iter()
        .zip(values)
        .try_for_each(|(file, value)| serialize(file, value, opts))
}

fn print_completions(cmd: &mut Command, shell: Shell) {
    generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() -> Result<()> {
    let opts = Options::parse();

    if let Some(shell) = opts.generate_completion {
        let mut cmd = Options::command();
        print_completions(&mut cmd, shell);
        std::process::exit(0);
    }

    #[cfg(feature = "color")]
    if opts.list_themes {
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

    if sources.is_empty() && !io::stdin().is_terminal() {
        // Input is piped on stdin.
        sources.push(Source::Stdin);
    }

    let sinks = opts.sinks;

    // Validate sinks to prevent accidentally overwriting existing files.
    for sink in &sinks {
        if let Sink::Path(path) = sink {
            if !path.exists() {
                continue;
            }

            if !path.is_file() {
                return Err(anyhow!(
                    "output file `{}` exists but is not a file",
                    path.display()
                ));
            } else if !opts.output.overwrite {
                return Err(anyhow!(
                    "output file `{}` exists, pass --overwrite to overwrite it",
                    path.display()
                ));
            }
        }
    }

    let value = match (sources.len(), dir_sources) {
        (0, false) => return Err(anyhow!("input file or data on stdin expected")),
        (1, false) => deserialize(&sources[0], &opts.input)?,
        (_, _) => deserialize_many(&sources, &opts.input)?,
    };

    let value = transform(value, &opts.transform)?;

    if sinks.len() <= 1 {
        serialize(sinks.first().unwrap_or(&Sink::Stdout), value, &opts.output)
    } else {
        serialize_many(&sinks, value, &opts.output)
    }
}
