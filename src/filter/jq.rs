//! A wrapper for `jq`.

use crate::{Error, Result};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;

/// A wrapper for the `jq` command.
///
/// This can be used to transform a `Value` using a `jq` expression.
///
/// ## Example
///
/// ```
/// use dts::filter::{jq, Filter};
/// use serde_json::json;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!([5, 4, 10]);
///
/// let jq = jq::Filter::parse("map(select(. > 5))")?;
/// let result = jq.apply(value)?;
///
/// assert_eq!(result, json!([10]));
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Filter {
    expr: String,
    executable: PathBuf,
}

impl Filter {
    /// Creates a new `Jq` instance.
    ///
    /// ## Errors
    ///
    /// If the `jq` executable cannot be found in the `PATH` or is invalid an error is returned.
    pub fn new(expr: &str) -> Result<Filter> {
        let exe = std::env::var("DTS_JQ")
            .ok()
            .unwrap_or_else(|| String::from("jq"));
        Filter::with_executable(expr, exe)
    }

    /// Creates a new `Jq` instance using the provided executable.
    ///
    /// ## Errors
    ///
    /// If `executable` cannot be found in `PATH`, does not exist (if absolute) or is invalid an
    /// error is returned.
    pub fn with_executable<P>(expr: &str, executable: P) -> Result<Filter>
    where
        P: AsRef<Path>,
    {
        let executable = executable.as_ref();

        let output = Command::new(executable)
            .arg("--version")
            .output()
            .map_err(|err| {
                if let io::ErrorKind::NotFound = err.kind() {
                    Error::new(format!("executable `{}` not found", executable.display()))
                } else {
                    Error::Io(err)
                }
            })?;

        let executable = executable.to_path_buf();
        let version = String::from_utf8_lossy(&output.stdout);

        if version.starts_with("jq-") {
            Ok(Filter {
                expr: expr.to_owned(),
                executable,
            })
        } else {
            Err(Error::new(format!(
                "executable `{}` exists but does appear to be `jq`",
                executable.display()
            )))
        }
    }

    fn spawn_cmd(&self) -> io::Result<Child> {
        Command::new(&self.executable)
            .arg("--compact-output")
            .arg("--monochrome-output")
            .arg(&self.expr)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

fn process_output(buf: &[u8]) -> Result<Value, Error> {
    let mut values = buf
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()))
        .collect::<Result<Vec<Value>, _>>()?;

    if values.len() == 1 {
        Ok(values.remove(0))
    } else {
        Ok(Value::Array(values))
    }
}

impl super::Filter for Filter {
    type Item = Filter;

    fn parse(expr: &str) -> Result<Self::Item> {
        Filter::new(expr)
    }

    fn apply(&self, value: Value) -> Result<Value> {
        let mut cmd = self.spawn_cmd()?;
        let mut stdin = cmd.stdin.take().unwrap();

        let buf = serde_json::to_vec(&value)?;

        thread::spawn(move || stdin.write_all(&buf));

        let output = cmd.wait_with_output()?;

        if output.status.success() {
            process_output(&output.stdout)
        } else {
            Err(Error::new(String::from_utf8_lossy(&output.stderr)))
        }
    }
}
