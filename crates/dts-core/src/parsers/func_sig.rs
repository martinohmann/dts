//! A parser for function signatures.

use super::{ParseError, ParseErrorKind};
use crate::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParseTrait;
use pest_derive::Parser;
use std::fmt;

#[derive(Parser)]
#[grammar = "parsers/grammars/func_sig.pest"]
struct FuncSigParser;

/// Parses function calls from a `&str`.
pub fn parse(input: &str) -> Result<Vec<FuncSig>, ParseError> {
    let funcs = FuncSigParser::parse(Rule::Root, input)
        .map_err(|e| ParseError::new(ParseErrorKind::FuncSig, e))?
        .into_iter()
        .filter_map(|pair| match pair.as_rule() {
            Rule::FuncSig => Some(parse_func_sig(pair.into_inner())),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect();

    Ok(funcs)
}

fn parse_func_sig(mut pairs: Pairs<Rule>) -> FuncSig {
    let name = pairs.next().unwrap().as_str();

    let args = pairs
        .next()
        .map(|pair| parse_func_args(pair.into_inner()))
        .unwrap_or_default();

    FuncSig::new(name, args)
}

fn parse_func_args(pairs: Pairs<Rule>) -> Vec<FuncArg> {
    pairs.map(parse_func_arg).collect()
}

fn parse_func_arg(pair: Pair<Rule>) -> FuncArg {
    let rule = pair.as_rule();
    let mut inner = pair.into_inner();

    match rule {
        Rule::PositionalArg => {
            let value = inner.next().unwrap().as_str();
            FuncArg::Positional(value)
        }
        Rule::NamedArg => {
            let name = inner.next().unwrap().as_str();
            let value = inner.next().unwrap().as_str();
            FuncArg::Named(name, value)
        }
        Rule::LegacyArg => {
            let value = inner.next().unwrap().as_str();
            FuncArg::Positional(value)
        }
        _ => unreachable!(),
    }
}

/// Represents a function call with arguments.
#[derive(Debug, PartialEq)]
pub struct FuncSig<'a> {
    name: &'a str,
    args: Vec<FuncArg<'a>>,
}

impl<'a> FuncSig<'a> {
    /// Creates a new `FuncSig` with name and arguments.
    pub fn new<I>(name: &'a str, args: I) -> Self
    where
        I: IntoIterator<Item = FuncArg<'a>>,
    {
        FuncSig {
            name,
            args: args.into_iter().collect(),
        }
    }

    /// Returns the function name.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns a reference to the function arguments.
    pub fn args(&self) -> &Vec<FuncArg<'a>> {
        &self.args
    }
}

impl<'a> fmt::Display for FuncSig<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Represents a function argument.
#[derive(Debug, PartialEq)]
pub enum FuncArg<'a> {
    /// Represents a named argument (e.g. `name=value`).
    Named(&'a str, &'a str),
    /// Represents a positional argument (e.g. `value`).
    Positional(&'a str),
}

impl<'a> fmt::Display for FuncArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncArg::Named(name, value) => write!(f, "{}=\"{}\"", name, value),
            FuncArg::Positional(value) => write!(f, "\"{}\"", value),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        assert!(parse("foo.[").is_err());
        assert_eq!(
            parse("foo").unwrap(),
            vec![FuncSig {
                name: "foo",
                args: Vec::new()
            }]
        );
        assert_eq!(
            parse("foo()").unwrap(),
            vec![FuncSig {
                name: "foo",
                args: Vec::new()
            }]
        );
        assert_eq!(
            parse("foo(bar)").unwrap(),
            vec![FuncSig {
                name: "foo",
                args: vec![FuncArg::Positional("bar")],
            }]
        );
        assert_eq!(
            parse("foo(\"bar\")").unwrap(),
            vec![FuncSig {
                name: "foo",
                args: vec![FuncArg::Positional("bar")],
            }]
        );
        assert_eq!(
            parse("foo(\"bar\", other = qux)").unwrap(),
            vec![FuncSig {
                name: "foo",
                args: vec![FuncArg::Positional("bar"), FuncArg::Named("other", "qux")]
            }]
        );

        assert_eq!(
            parse("foo(), bar; baz(qux)").unwrap(),
            vec![
                FuncSig {
                    name: "foo",
                    args: Vec::new()
                },
                FuncSig {
                    name: "bar",
                    args: Vec::new()
                },
                FuncSig {
                    name: "baz",
                    args: vec![FuncArg::Positional("qux")]
                }
            ]
        );
    }

    #[test]
    fn test_parse_legacy() {
        assert_eq!(
            parse("foo='bar',bar=baz:2,qux=\"one\":\'two\':3").unwrap(),
            vec![
                FuncSig {
                    name: "foo",
                    args: vec![FuncArg::Positional("bar")]
                },
                FuncSig {
                    name: "bar",
                    args: vec![FuncArg::Positional("baz"), FuncArg::Positional("2")]
                },
                FuncSig {
                    name: "qux",
                    args: vec![
                        FuncArg::Positional("one"),
                        FuncArg::Positional("two"),
                        FuncArg::Positional("3")
                    ]
                }
            ]
        );
    }
}
