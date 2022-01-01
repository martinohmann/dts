use super::{ParseError, ParseErrorKind};
use crate::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParseTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parsers/grammars/funcs.pest"]
struct FuncParser;

/// Parses function calls from a `&str`.
pub fn parse<'a>(input: &'a str) -> Result<Vec<Func<'a>>, ParseError> {
    let funcs = FuncParser::parse(Rule::Root, input)
        .map_err(|e| ParseError::new(ParseErrorKind::Func, e))?
        .into_iter()
        .filter_map(|pair| match pair.as_rule() {
            Rule::FuncCall => Some(parse_func(pair.into_inner())),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect();

    Ok(funcs)
}

fn parse_func<'a>(mut pairs: Pairs<'a, Rule>) -> Func<'a> {
    let name = pairs.next().unwrap().as_str();
    let args = match pairs.next() {
        Some(args) => parse_func_args(args.into_inner()),
        None => Vec::new(),
    };

    Func { name, args }
}

fn parse_func_args<'a>(pairs: Pairs<'a, Rule>) -> Vec<FuncArg<'a>> {
    pairs.map(parse_func_arg).collect()
}

fn parse_func_arg<'a>(pair: Pair<'a, Rule>) -> FuncArg<'a> {
    match pair.as_rule() {
        Rule::PositionalArg => {
            let value = pair.into_inner().next().unwrap().as_str();
            FuncArg::Positional(value)
        }
        Rule::NamedArg => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str();
            let value = inner.next().unwrap().as_str();
            FuncArg::Named(name, value)
        }
        _ => unreachable!(),
    }
}

/// Represents a function call with arguments.
#[derive(Debug, PartialEq)]
pub struct Func<'a> {
    /// The function name.
    pub name: &'a str,
    /// The list of function arguments.
    pub args: Vec<FuncArg<'a>>,
}

/// Represents a function argument.
#[derive(Debug, PartialEq)]
pub enum FuncArg<'a> {
    /// Represents a named argument (e.g. `name=value`).
    Named(&'a str, &'a str),
    /// Represents a positional argument (e.g. `value`).
    Positional(&'a str),
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
            vec![Func {
                name: "foo",
                args: Vec::new()
            }]
        );
        assert_eq!(
            parse("foo()").unwrap(),
            vec![Func {
                name: "foo",
                args: Vec::new()
            }]
        );
        assert_eq!(
            parse("foo(bar)").unwrap(),
            vec![Func {
                name: "foo",
                args: vec![FuncArg::Positional("bar")],
            }]
        );
        assert_eq!(
            parse("foo(\"bar\")").unwrap(),
            vec![Func {
                name: "foo",
                args: vec![FuncArg::Positional("bar")],
            }]
        );
        assert_eq!(
            parse("foo(\"bar\", other = qux)").unwrap(),
            vec![Func {
                name: "foo",
                args: vec![FuncArg::Positional("bar"), FuncArg::Named("other", "qux")]
            }]
        );

        assert_eq!(
            parse("foo(), bar; baz(qux)").unwrap(),
            vec![
                Func {
                    name: "foo",
                    args: Vec::new()
                },
                Func {
                    name: "bar",
                    args: Vec::new()
                },
                Func {
                    name: "baz",
                    args: vec![FuncArg::Positional("qux")]
                }
            ]
        );
    }
}
