//! dts is a simple command line tool to transcode between different input and output encodings.

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;
use crossbeam_utils::thread;
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
        .context(format!("error in {}", path.display()))
        .context(format!("failed to deserialize {}", encoding))
}

fn deserialize_parallel(paths: &[PathBuf], opts: &InputOptions) -> Result<Value> {
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
                    // Propagate the index down the result channel.
                    let result = deserialize(path, opts).map(|value| (index, value));

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
    results.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Value::Array(
        results.iter().map(|(_, v)| v.clone()).collect(),
    ))
}

fn deserialize_many(paths: &[PathBuf], opts: &InputOptions) -> Result<Value> {
    if opts.threads > 1 && paths.len() > 1 {
        deserialize_parallel(paths, opts)
    } else {
        Ok(Value::Array(
            paths
                .iter()
                .map(|path| deserialize(path, opts))
                .collect::<Result<Vec<_>>>()?,
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
        .context(format!("failed to serialize {}", encoding))
}

fn glob_dir(path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
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

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut paths = Vec::with_capacity(opts.paths.len());

    if !atty::is(atty::Stream::Stdin) {
        // Input is piped on stdin.
        paths.push(PathBuf::from("-"));
    }

    for path in &opts.paths {
        if path.is_file() {
            paths.push(path.clone());
        } else {
            match &opts.input.glob {
                Some(pattern) => {
                    let mut matches = glob_dir(path, pattern)?;
                    paths.append(&mut matches);
                }
                None => {
                    return Err(anyhow!(
                        "--glob is required if input paths contain directories"
                    ))
                }
            }
        }
    }

    let value = match paths.len() {
        0 => return Err(anyhow!("input file or data on stdin expected")),
        1 => deserialize(&paths[0], &opts.input)?,
        _ => deserialize_many(&paths, &opts.input)?,
    };

    let value = transform(&value, &opts.transform)?;

    serialize(&value, &opts.output)
}
