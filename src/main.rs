//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use indexmap::IndexMap;
use serde_json::Map;
use std::fs::canonicalize;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use dts::{
    args::{InputOptions, Options, OutputOptions, TransformOptions},
    de::Deserializer,
    detect_encoding,
    io::{Reader, Writer},
    ser::Serializer,
    transform, Encoding, Value,
};

fn deserialize(path: &Path, opts: &InputOptions) -> Result<Value> {
    let encoding = detect_encoding(opts.input_encoding, path)
        .context("unable to detect input encoding, please provide it explicitly via -i")?;

    let reader = Reader::new(path)
        .with_context(|| format!("failed to open input file: {}", path.display()))?;
    let mut de = Deserializer::with_options(BufReader::new(reader), opts.into());

    de.deserialize(encoding)
        .with_context(|| format!("error in {}", path.display()))
        .with_context(|| format!("failed to deserialize {}", encoding))
}

struct DeserializeResult<'a> {
    index: usize,
    path: &'a PathBuf,
    value: Value,
}

fn deserialize_many(paths: &[PathBuf], opts: &InputOptions) -> Result<Value> {
    // We need minimum one worker and max paths.len().
    let workers = paths.len().min(opts.threads).max(1);

    let (tx_res, rx_res) = bounded(paths.len());

    thread::scope(|scope| {
        let (tx_paths, rx_paths) = bounded(paths.len());

        scope.spawn(move |_| {
            for (index, path) in paths.iter().enumerate() {
                // Send the index down the channel along with the paths so that we can order
                // results later after collecting them.
                if tx_paths.send((index, path)).is_err() {
                    break;
                }
            }
        });

        for _ in 0..workers {
            let (tx_res, rx_paths) = (tx_res.clone(), rx_paths.clone());

            scope.spawn(move |_| {
                for (index, path) in rx_paths.iter() {
                    // Propagate the index and path down the result channel.
                    let result = deserialize(path, opts).map(|value| DeserializeResult {
                        index,
                        path,
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
        let cwd = std::env::current_dir()?;

        let map = results
            .iter()
            .map(|result| {
                Ok((
                    relative_path(result.path, &cwd)?
                        .to_string_lossy()
                        .to_string(),
                    result.value.clone(),
                ))
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(Value::Object(Map::from_iter(map)))
    } else {
        Ok(Value::Array(
            results.iter().map(|result| result.value.clone()).collect(),
        ))
    }
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
    let mut ser = Serializer::with_options(BufWriter::new(writer), opts.into());

    ser.serialize(encoding, value)
        .with_context(|| format!("failed to serialize {}", encoding))
}

fn glob_dir(path: &Path, opts: &InputOptions) -> Result<Vec<PathBuf>> {
    match &opts.glob {
        Some(pattern) => {
            let matches = glob::glob(&path.join(pattern).to_string_lossy())
                .context("invalid glob pattern")?
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
        None => Err(anyhow!(
            "--glob is required if input paths contain directories"
        )),
    }
}

fn relative_path(path: &Path, base: &Path) -> Result<PathBuf> {
    let path = canonicalize(path)?;
    let base = canonicalize(base)?;

    pathdiff::diff_paths(&path, &base).ok_or_else(|| {
        anyhow!(
            "failed to calculate path diff between {} and {}",
            path.display(),
            base.display()
        )
    })
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut paths = Vec::with_capacity(opts.paths.len());

    if !atty::is(atty::Stream::Stdin) {
        // Input is piped on stdin.
        paths.push(PathBuf::from("-"));
    }

    let mut force_collection = false;

    for path in &opts.paths {
        if !path.exists() {
            return Err(anyhow!(
                "file or directory does not exist: {}",
                path.display()
            ));
        } else if path.is_file() {
            paths.push(path.clone());
        } else {
            // Force deserialization into a collection (array or object with file paths as keys
            // depending on the input options) even if all directory globs only produces a single
            // file path. This will ensure that deserializing the files that resulted from
            // directory globs always produces a consistent structure of the data.
            force_collection = true;

            let mut matches = glob_dir(path, &opts.input)?;
            paths.append(&mut matches);
        }
    }

    let value = match paths.len() {
        0 => return Err(anyhow!("input file or data on stdin expected")),
        1 if !force_collection => deserialize(&paths[0], &opts.input)?,
        _ => deserialize_many(&paths, &opts.input)?,
    };

    let value = transform(&value, &opts.transform)?;

    serialize(&value, &opts.output)
}
