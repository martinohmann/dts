//! A wrapper for `jaq`.

use crate::{Error, Result};
use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
struct ParseError {
    expr: String,
    errs: Vec<jaq_parse::Error>,
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

pub(crate) struct Filter {
    filter: jaq_interpret::Filter,
}

impl Filter {
    pub(crate) fn new(expr: &str) -> Result<Filter> {
        let mut defs = ParseCtx::new(Vec::new());
        defs.insert_natives(jaq_core::core());
        defs.insert_defs(jaq_std::std());

        let (main, errs) = jaq_parse::parse(expr, jaq_parse::main());

        if errs.is_empty() {
            Ok(Filter {
                filter: defs.compile(main.unwrap()),
            })
        } else {
            Err(Error::new(ParseError {
                expr: expr.to_owned(),
                errs,
            }))
        }
    }

    pub(crate) fn apply(&self, value: Value) -> Result<Value> {
        let empty: Vec<Result<Val, String>> = Vec::new();
        let iter = RcIter::new(empty.into_iter());
        let mut values = self
            .filter
            .run((Ctx::new(Vec::new(), &iter), Val::from(value)))
            .map(|out| Ok(Value::from(out.map_err(Error::new)?)))
            .collect::<Result<Vec<_>, Error>>()?;

        if values.len() == 1 {
            Ok(values.remove(0))
        } else {
            Ok(Value::Array(values))
        }
    }
}
