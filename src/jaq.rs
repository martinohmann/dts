//! A wrapper for `jaq`.

use crate::{Error, Result};
use jaq_core::{Definitions, Filter, Val};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
struct ParseError {
    filter: String,
    errs: Vec<jaq_core::parse::Error>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid filter `{}`: ", self.filter)?;

        for (i, err) in self.errs.iter().enumerate() {
            if i > 0 {
                write!(f, "; {}", err)?;
            } else {
                write!(f, "{}", err)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// A wrapper for a `jaq` filter.
///
/// This can be used to transform a `Value` using a `jq`-like expression.
pub struct Jaq {
    filter: Filter,
}

impl Jaq {
    /// Parses the filter and creates a new `Jaq` instance if the filter is valid.
    pub fn parse(filter: &str) -> Result<Jaq> {
        let mut errs = Vec::new();
        let mut defs = Definitions::core();

        jaq_std::std()
            .into_iter()
            .for_each(|def| defs.insert(def, &mut errs));

        assert!(errs.is_empty());

        let (main, mut errs) = jaq_core::parse::parse(filter, jaq_core::parse::main());
        let f = main.map(|main| defs.finish(main, &mut errs));

        if errs.is_empty() {
            Ok(Jaq { filter: f.unwrap() })
        } else {
            Err(Error::new(ParseError {
                filter: filter.to_owned(),
                errs,
            }))
        }
    }

    /// Processes a `Value` and returns the result.
    pub fn process(&self, input: Value) -> Result<Value> {
        self.filter
            .run(Val::from(input))
            .map(|out| Ok(Value::from(out.map_err(Error::new)?)))
            .collect::<Result<Vec<Value>, Error>>()
            .map(Value::Array)
    }
}
