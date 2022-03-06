//! A wrapper for `jq`.

use crate::Result;
use jq_rs::{self, JqProgram};
use serde_json::Value;
use std::cell::RefCell;
use std::io::BufRead;
use std::path::Path;

/// A wrapper for the `jq` command.
///
/// This can be used to transform a `Value` using a `jq` expression.
///
/// ## Example
///
/// ```
/// use dts_core::jq::Jq;
/// use serde_json::json;
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let value = json!([5, 4, 10]);
///
/// let jq = Jq::new("map(select(. > 5))")?;
/// let result = jq.process(&value)?;
///
/// assert_eq!(result, json!([10]));
/// #   Ok(())
/// # }
/// ```
pub struct Jq {
    program: RefCell<JqProgram>,
}

impl Jq {
    /// Creates a new `Jq` instance which uses the given expression.
    pub fn new(expr: &str) -> Result<Jq> {
        let program = jq_rs::compile(expr)?;
        Ok(Jq {
            program: RefCell::new(program),
        })
    }

    /// Creates a new `Jq` instance which uses an expression from a file.
    pub fn from_path<P>(path: P) -> Result<Jq>
    where
        P: AsRef<Path>,
    {
        let expr = std::fs::read_to_string(path)?;
        Jq::new(&expr)
    }

    /// Processes a `Value` and returns the result.
    pub fn process(&self, value: &Value) -> Result<Value> {
        let data = serde_json::to_string(value)?;
        let output = self.program.borrow_mut().run(&data)?;

        process_output(output.as_bytes())
    }
}

fn process_output(buf: &[u8]) -> Result<Value> {
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
