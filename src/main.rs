//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use serde_json::Map;
use std::io::{BufReader, BufWriter};

use dts::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    de::Deserializer,
    ser::Serializer,
    transform, Encoding, Error, Sink, Source, Value,
};

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
        .with_context(|| format!("Failed to deserialize `{}`", encoding))
}

struct DeserializeResult<'a> {
    index: usize,
    source: &'a Source,
    value: Value,
}

fn deserialize_many(sources: &[Source], opts: &InputOptions) -> Result<Value> {
    // We need minimum one worker and max sources.len().
    let workers = sources.len().min(opts.threads).max(1);

    let (tx_res, rx_res) = bounded(sources.len());

    thread::scope(|scope| {
        let (tx_sources, rx_sources) = bounded(sources.len());

        scope.spawn(move |_| {
            for (index, source) in sources.iter().enumerate() {
                // Send the index down the channel along with the source so that we can order
                // results later after collecting them.
                if tx_sources.send((index, source)).is_err() {
                    break;
                }
            }
        });

        for _ in 0..workers {
            let (tx_res, rx_sources) = (tx_res.clone(), rx_sources.clone());

            scope.spawn(move |_| {
                for (index, source) in rx_sources.iter() {
                    // Propagate the index and source down the result channel.
                    let result = deserialize(source, opts).map(|value| DeserializeResult {
                        index,
                        source,
                        value,
                    });

                    if tx_res.send(result).is_err() {
                        break;
                    }
                }
            });
        }
    })
    .unwrap();

    // Drop the sender so we can collect the results.
    drop(tx_res);

    let mut results = rx_res.iter().collect::<Result<Vec<_>>>()?;

    // Sort by path index to restore the original order.
    results.sort_by(|a, b| a.index.cmp(&b.index));

    if opts.file_paths {
        let iter = results
            .iter()
            .map(|res| (res.source.to_string(), res.value.clone()));

        Ok(Value::Object(Map::from_iter(iter)))
    } else {
        let iter = results.iter().map(|res| res.value.clone());

        Ok(Value::Array(Vec::from_iter(iter)))
    }
}

fn transform(value: &Value, opts: &TransformOptions) -> Result<Value> {
    transform::apply_chain(&opts.transform, value).context("Failed to transform value")
}

fn serialize(sink: &Sink, value: &Value, opts: &OutputOptions) -> Result<()> {
    let encoding = opts
        .output_encoding
        .or_else(|| sink.encoding())
        .unwrap_or(Encoding::JSON);

    let writer = sink
        .to_writer()
        .with_context(|| format!("Failed to create writer for sink `{}`", sink))?;

    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    match ser.serialize(encoding, value) {
        Ok(()) => Ok(()),
        Err(Error::IOError(e)) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err),
    }
    .with_context(|| format!("Failed to serialize `{}`", encoding))
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

fn main() -> Result<()> {
    let opts = Options::parse();

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

    let mut value = transform(&value, &opts.transform)?;

    let sinks = opts.sinks;

    if sinks.len() <= 1 {
        serialize(sinks.get(0).unwrap_or(&Sink::Stdout), &value, &opts.output)
    } else {
        serialize_many(&sinks, &mut value, &opts.output)
    }
}
