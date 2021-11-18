//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use serde_json::Map;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use dts::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    de::Deserializer,
    ser::Serializer,
    transform, Encoding, Error, Source, Value,
};

fn deserialize(source: &Source, opts: &InputOptions) -> Result<Value> {
    let encoding = opts
        .input_encoding
        .or_else(|| source.encoding())
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let reader = source
        .to_reader()
        .with_context(|| format!("failed to create reader for source: {}", source))?;

    let mut de = Deserializer::with_options(BufReader::new(reader), opts.into());

    de.deserialize(encoding)
        .with_context(|| format!("error in source: {}", source))
        .with_context(|| format!("failed to deserialize {}", encoding))
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
    transform::apply_chain(&opts.transform, value).context("failed to transform value")
}

fn serialize(value: &Value, opts: &OutputOptions) -> Result<()> {
    let sink = &opts.output_file;

    let encoding = opts
        .output_encoding
        .or_else(|| sink.encoding())
        .unwrap_or(Encoding::Json);

    let writer = sink
        .to_writer()
        .with_context(|| format!("failed to create writer for sink: {}", sink))?;

    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    match ser.serialize(encoding, value) {
        Ok(()) => Ok(()),
        Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err),
    }
    .with_context(|| format!("failed to serialize {}", encoding))
}

fn glob_dir(path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let matches = glob::glob(&path.join(pattern).to_string_lossy())?
        .filter_map(|entry| match entry {
            Ok(path) => match path.is_file() {
                true => Some(Ok(path)),
                false => None,
            },
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(matches)
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut sources = Vec::with_capacity(opts.sources.len());

    if !atty::is(atty::Stream::Stdin) {
        // Input is piped on stdin.
        sources.push(Source::Stdin);
    }

    let mut force_collection = false;

    for source in &opts.sources {
        match source.as_path() {
            Some(path) => {
                if !path.exists() {
                    return Err(anyhow!("file or directory does not exist: {}", source));
                } else if path.is_file() {
                    sources.push(path.into());
                } else {
                    let pattern = opts
                        .input
                        .glob
                        .as_ref()
                        .context("--glob is required if sources contain directories")?;

                    // Force deserialization into a collection (array or object with file paths as keys
                    // depending on the input options) even if all directory globs only produces a single
                    // file path. This will ensure that deserializing the files that resulted from
                    // directory globs always produces a consistent structure of the data.
                    force_collection = true;

                    for path in glob_dir(path, pattern).context("invalid glob pattern")? {
                        sources.push(path.as_path().into());
                    }
                }
            }
            None => sources.push(source.clone()),
        }
    }

    let value = match sources.len() {
        0 => return Err(anyhow!("input file or data on stdin expected")),
        1 if !force_collection => deserialize(&sources[0], &opts.input)?,
        _ => deserialize_many(&sources, &opts.input)?,
    };

    let value = transform(&value, &opts.transform)?;

    serialize(&value, &opts.output)
}
