//! A wrapper for `jaq`.

use crate::{Error, Result};
use jaq_core::{self, Definitions, Val};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
struct ParseError {
    expr: String,
    errs: Vec<jaq_core::parse::Error>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid filter expression `{}`: ", self.expr)?;

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
pub(crate) struct Filter {
    filter: jaq_core::Filter,
}

impl Filter {
    pub(crate) fn new(expr: &str) -> Result<Filter> {
        let mut errs = Vec::new();
        let mut defs = Definitions::core();

        jaq_std::std()
            .into_iter()
            .for_each(|def| defs.insert(def, &mut errs));

        assert!(errs.is_empty());

        let (main, mut errs) = jaq_core::parse::parse(expr, jaq_core::parse::main());
        let f = main.map(|main| defs.finish(main, &mut errs));

        if errs.is_empty() {
            Ok(Filter { filter: f.unwrap() })
        } else {
            Err(Error::new(ParseError {
                expr: expr.to_owned(),
                errs,
            }))
        }
    }

    pub(crate) fn apply(&self, value: Value) -> Result<Value> {
        let mut values = self
            .filter
            .run(Val::from(value))
            .map(|out| Ok(Value::from(out.map_err(Error::new)?)))
            .collect::<Result<Vec<_>, Error>>()?;

        if values.len() == 1 {
            Ok(values.remove(0))
        } else {
            Ok(Value::Array(values))
        }
    }
}
