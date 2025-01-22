//! A wrapper for `jaq`.

use crate::{Error, Result};
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{compile, load, Compiler, Ctx, Native, RcIter};
use jaq_json::Val;
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
struct ParseError {
    expr: String,
    errs: Vec<String>,
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
    filter: jaq_core::Filter<Native<Val>>,
}

impl Filter {
    pub(crate) fn new(expr: &str) -> Result<Filter> {
        let program = File {
            code: expr,
            path: (),
        };

        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();

        let modules = loader.load(&arena, program).map_err(|errs| {
            Error::new(ParseError {
                expr: expr.to_owned(),
                errs: load_errors(errs),
            })
        })?;

        let filter = Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|errs| {
                Error::new(ParseError {
                    expr: expr.to_owned(),
                    errs: compile_errors(errs),
                })
            })?;

        Ok(Filter { filter })
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

fn load_errors(errs: load::Errors<&str, ()>) -> Vec<String> {
    errs.into_iter()
        .map(|(_, err)| match err {
            load::Error::Io(errs) => errs.into_iter().map(report_io).collect(),
            load::Error::Lex(errs) => errs.into_iter().map(report_lex).collect(),
            load::Error::Parse(errs) => errs.into_iter().map(report_parse).collect(),
        })
        .collect()
}

fn compile_errors(errs: compile::Errors<&str, ()>) -> Vec<String> {
    errs.into_iter()
        .flat_map(|(_, errs)| errs.into_iter().map(report_compile))
        .collect()
}

fn report_io((path, error): (&str, String)) -> String {
    format!("could not load file {}: {}", path, error)
}

fn report_lex((expected, _found): load::lex::Error<&str>) -> String {
    format!("expected {}", expected.as_str())
}

fn report_parse((expected, _found): load::parse::Error<&str>) -> String {
    format!("expected {}", expected.as_str())
}

fn report_compile((found, undefined): compile::Error<&str>) -> String {
    let wnoa = |exp, got| format!("wrong number of arguments (expected {exp}, found {got})");
    match (found, undefined) {
        ("reduce", compile::Undefined::Filter(arity)) => wnoa("2", arity),
        ("foreach", compile::Undefined::Filter(arity)) => wnoa("2 or 3", arity),
        (_, undefined) => format!("undefined {}", undefined.as_str()),
    }
}
