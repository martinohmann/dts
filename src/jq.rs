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
/// use dts::jq::Jq;
/// use serde_json::json;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!([5, 4, 10]);
///
/// let jq = Jq::new()?;
/// let result = jq.process("map(select(. > 5))", &value)?;
///
/// assert_eq!(result, json!([10]));
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Jq {
    executable: PathBuf,
}

impl Jq {
    /// Creates a new `Jq` instance.
    ///
    /// ## Errors
    ///
    /// If the `jq` executable cannot be found in the `PATH` or is invalid an error is returned.
    pub fn new() -> Result<Jq> {
        Jq::with_executable("jq")
    }

    /// Creates a new `Jq` instance using the provided executable.
    ///
    /// ## Errors
    ///
    /// If `executable` cannot be found in `PATH`, does not exist (if absolute) or is invalid an
    /// error is returned.
    pub fn with_executable<P>(executable: P) -> Result<Jq>
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
            Ok(Jq { executable })
        } else {
            Err(Error::new(format!(
                "executable `{}` exists but does appear to be `jq`",
                executable.display()
            )))
        }
    }

    /// Processes a `Value` using the provided jq expression and returns the result.
    ///
    /// ## Errors
    ///
    /// - `Error::Io` if spawning `jq` fails or if there are other I/O errors.
    /// - `Error::Json` if the data returned by `jq` cannot be deserialized.
    /// - `Error::Message` on any other error.
    pub fn process(&self, expr: &str, value: &Value) -> Result<Value> {
        let mut cmd = self.spawn_cmd(expr)?;
        let mut stdin = cmd.stdin.take().unwrap();

        let buf = serde_json::to_vec(value)?;

        thread::spawn(move || stdin.write_all(&buf));

        let output = cmd.wait_with_output()?;

        if output.status.success() {
            process_output(&output.stdout)
        } else {
            Err(Error::new(String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn spawn_cmd(&self, expr: &str) -> io::Result<Child> {
        Command::new(&self.executable)
            .arg("--compact-output")
            .arg("--monochrome-output")
            .arg(expr)
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
