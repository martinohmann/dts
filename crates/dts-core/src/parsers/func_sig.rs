//! A parser for function signatures.

use super::{ParseError, ParseErrorKind};
use crate::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParseTrait;
use pest_derive::Parser;
use std::fmt::{self, Write};

#[derive(Parser)]
#[grammar = "parsers/grammars/func_sig.pest"]
struct FuncSigParser;

/// Represents an expression.
#[derive(Debug, PartialEq)]
pub enum ExprTerm<'a> {
    /// An expression consisting of one or more function signatures.
    Expr(Vec<FuncSig<'a>>),
    /// A value.
    Value(&'a str),
}

impl<'a> fmt::Display for ExprTerm<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprTerm::Value(value) => write!(f, "\"{}\"", value),
            ExprTerm::Expr(sigs) => {
                for (i, sig) in sigs.iter().enumerate() {
                    if i > 0 {
                        f.write_char('.')?;
                    }

                    f.write_str(&sig.to_string())?;
                }

                Ok(())
            }
        }
    }
}

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
            let expr_term = parse_expr_term(inner.next().unwrap());
            FuncArg::Positional(expr_term)
        }
        Rule::NamedArg => {
            let name = inner.next().unwrap().as_str();
            let expr_term = parse_expr_term(inner.next().unwrap());
            FuncArg::Named(name, expr_term)
        }
        _ => unreachable!(),
    }
}

fn parse_expr_term(pair: Pair<Rule>) -> ExprTerm {
    let pair = pair.into_inner().next().unwrap();
    let rule = pair.as_rule();
    let mut inner = pair.into_inner();

    match rule {
        Rule::Value => ExprTerm::Value(inner.next().unwrap().as_str()),
        Rule::Expr => {
            let func_sigs = inner
                .map(|pair| parse_func_sig(pair.into_inner()))
                .collect();
            ExprTerm::Expr(func_sigs)
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
    Named(&'a str, ExprTerm<'a>),
    /// Represents a positional argument (e.g. `value`).
    Positional(ExprTerm<'a>),
}

impl<'a> fmt::Display for FuncArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncArg::Named(name, value) => write!(f, "{}={}", name, value),
            FuncArg::Positional(value) => write!(f, "\"{}\"", value),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[track_caller]
    fn assert_parse(s: &str, expected: Vec<FuncSig>) {
        assert_eq!(parse(s).unwrap(), expected);
    }

    #[test]
    fn test_parse_simple() {
        assert_parse("foo", vec![FuncSig::new("foo", vec![])]);
        assert_parse("foo()", vec![FuncSig::new("foo", vec![])]);
        assert_parse(
            "foo(1)",
            vec![FuncSig::new(
                "foo",
                vec![FuncArg::Positional(ExprTerm::Value("1"))],
            )],
        );
        assert_parse(
            "foo(-1.0e10)",
            vec![FuncSig::new(
                "foo",
                vec![FuncArg::Positional(ExprTerm::Value("-1.0e10"))],
            )],
        );
        assert_parse(
            "foo(true, false)",
            vec![FuncSig::new(
                "foo",
                vec![
                    FuncArg::Positional(ExprTerm::Value("true")),
                    FuncArg::Positional(ExprTerm::Value("false")),
                ],
            )],
        );
        assert_parse(
            "foo('bar')",
            vec![FuncSig::new(
                "foo",
                vec![FuncArg::Positional(ExprTerm::Value("bar"))],
            )],
        );
        assert_parse(
            "foo(\"bar\")",
            vec![FuncSig::new(
                "foo",
                vec![FuncArg::Positional(ExprTerm::Value("bar"))],
            )],
        );
    }

    #[test]
    fn test_parse_complex() {
        assert_parse(
            "foo(\"bar\", other = 'qux', three=4)",
            vec![FuncSig {
                name: "foo",
                args: vec![
                    FuncArg::Positional(ExprTerm::Value("bar")),
                    FuncArg::Named("other", ExprTerm::Value("qux")),
                    FuncArg::Named("three", ExprTerm::Value("4")),
                ],
            }],
        );

        assert_parse(
            "foo().bar baz('qux')",
            vec![
                FuncSig {
                    name: "foo",
                    args: Vec::new(),
                },
                FuncSig {
                    name: "bar",
                    args: Vec::new(),
                },
                FuncSig {
                    name: "baz",
                    args: vec![FuncArg::Positional(ExprTerm::Value("qux"))],
                },
            ],
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse("foo.[").is_err());
        assert!(parse("foo('baz)").is_err());
    }

    #[test]
    fn test_parse_expression() {
        assert_parse(
            "foo(bar, baz('qux', 1), qux = fn().other_fn())",
            vec![FuncSig::new(
                "foo",
                vec![
                    FuncArg::Positional(ExprTerm::Expr(vec![FuncSig::new("bar", vec![])])),
                    FuncArg::Positional(ExprTerm::Expr(vec![FuncSig::new(
                        "baz",
                        vec![
                            FuncArg::Positional(ExprTerm::Value("qux")),
                            FuncArg::Positional(ExprTerm::Value("1")),
                        ],
                    )])),
                    FuncArg::Named(
                        "qux",
                        ExprTerm::Expr(vec![
                            FuncSig::new("fn", vec![]),
                            FuncSig::new("other_fn", vec![]),
                        ]),
                    ),
                ],
            )],
        );
    }
}
