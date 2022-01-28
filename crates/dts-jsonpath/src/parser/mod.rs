//! Provides a jsonpath parser and types for the AST elements of a jsonpath query.

pub mod ast;

use crate::{path::CompOp, Error, Result};
use ast::*;
use dts_json::Value;
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParserTrait;
use pest_derive::Parser;
use regex::Regex;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/grammar/jsonpath.pest"]
struct JsonPathParser;

/// Parses a `JsonPath` from an input string or returns an error if the input is not a value
/// jsonpath query.
pub fn parse(input: &str) -> Result<Vec<Selector>> {
    let pairs = JsonPathParser::parse(Rule::Root, input)?;
    parse_jsonpath(pairs)
}

fn parse_jsonpath(pairs: Pairs<Rule>) -> Result<Vec<Selector>> {
    pairs
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(parse_selector)
        .collect()
}

fn parse_selector(pair: Pair<Rule>) -> Result<Selector> {
    match pair.as_rule() {
        Rule::RootSelector => Ok(Selector::Root),
        Rule::CurrentSelector => Ok(Selector::Current),
        Rule::DotSelector => Ok(Selector::Key(parse_string(pair))),
        Rule::DotWildSelector => Ok(Selector::Wildcard),
        Rule::IndexSelector => Ok(parse_index_selector(pair)),
        Rule::IndexWildSelector => Ok(Selector::IndexWildcard),
        Rule::UnionSelector => Ok(Selector::Union(parse_union_selector(pair))),
        Rule::SliceSelector => Ok(Selector::Slice(parse_slice_selector(pair))),
        Rule::DescendantSelector => Ok(Selector::Descendant(Box::new(parse_descendant_selector(
            pair,
        )))),
        Rule::FilterSelector => Ok(Selector::Filter(parse_filter_expr(inner(pair))?)),
        rule => unmatched_rule(rule),
    }
}

fn parse_index_selector(pair: Pair<Rule>) -> Selector {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::ElementIndex => Selector::Index(parse_int(pair)),
        Rule::QuotedMemberName => Selector::Key(parse_quoted_string(pair)),
        rule => unmatched_rule(rule),
    }
}

fn parse_descendant_selector(pair: Pair<Rule>) -> Selector {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::DotMemberName => Selector::Key(parse_string(pair)),
        Rule::IndexSelector => parse_index_selector(pair),
        Rule::IndexWildSelector => Selector::IndexWildcard,
        Rule::Wildcard => Selector::Wildcard,
        rule => unmatched_rule(rule),
    }
}

fn parse_slice_selector(pair: Pair<Rule>) -> SliceRange {
    pair.into_inner()
        .next()
        .map(parse_slice_range)
        .unwrap_or_default()
}

fn parse_slice_range(pair: Pair<Rule>) -> SliceRange {
    let mut slice = SliceRange::default();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::SliceStart => slice.start = pair.as_str().parse().ok(),
            Rule::SliceEnd => slice.end = pair.as_str().parse().ok(),
            Rule::SliceStep => slice.step = pair.as_str().parse().ok(),
            rule => unmatched_rule(rule),
        }
    }

    slice
}

fn parse_union_selector(pair: Pair<Rule>) -> Vec<Selector> {
    pair.into_inner()
        .map(|pair| {
            let pair = inner(pair);

            match pair.as_rule() {
                Rule::ElementIndex => Selector::Index(parse_int(pair)),
                Rule::QuotedMemberName => Selector::Key(parse_quoted_string(pair)),
                Rule::SliceIndex => Selector::Slice(parse_slice_range(pair)),
                rule => unmatched_rule(rule),
            }
        })
        .collect()
}

fn parse_filter_expr(pair: Pair<Rule>) -> Result<FilterExpr> {
    let expr = match pair.as_rule() {
        Rule::LogicalOrExpr => FilterExpr::Or(parse_filter_exprs(pair)?),
        Rule::LogicalAndExpr => FilterExpr::And(parse_filter_exprs(pair)?),
        Rule::ExistExpr => FilterExpr::Exist(parse_jsonpath(inner(pair).into_inner())?),
        Rule::NegExpr => FilterExpr::Not(Box::new(parse_filter_expr(inner(pair))?)),
        Rule::CompExpr => FilterExpr::Comp(parse_comp_expr(pair)?),
        Rule::RegexExpr => FilterExpr::Regex(parse_regex_expr(pair)?),
        Rule::ContainExpr => FilterExpr::Comp(parse_contain_expr(pair)?),
        rule => unmatched_rule(rule),
    };

    // Unwrap single expr or/and exprs.
    match expr {
        FilterExpr::Or(mut exprs) | FilterExpr::And(mut exprs) if exprs.len() == 1 => {
            Ok(exprs.swap_remove(0))
        }
        expr => Ok(expr),
    }
}

fn parse_filter_exprs(pair: Pair<Rule>) -> Result<Vec<FilterExpr>> {
    pair.into_inner()
        .map(parse_filter_expr)
        .collect::<Result<Vec<_>>>()
}

fn parse_regex_expr(pair: Pair<Rule>) -> Result<RegexExpr> {
    let mut pairs = pair.into_inner();

    Ok(RegexExpr {
        lhs: parse_regex_matchable(inner(pairs.next().unwrap()))?,
        regex: parse_regex(inner(pairs.next().unwrap()))?,
    })
}

fn parse_regex_matchable(pair: Pair<Rule>) -> Result<Operand> {
    match pair.as_rule() {
        Rule::String => Ok(Operand::Value(parse_quoted_string(pair).into())),
        Rule::RelPath | Rule::JsonPath => Ok(Operand::Path(parse_jsonpath(pair.into_inner())?)),
        rule => unmatched_rule(rule),
    }
}

fn parse_regex(pair: Pair<Rule>) -> Result<Regex> {
    Regex::new(pair.as_str()).map_err(Error::new)
}

fn parse_comp_expr(pair: Pair<Rule>) -> Result<CompExpr> {
    let mut pairs = pair.into_inner();

    Ok(CompExpr {
        lhs: parse_comparable(pairs.next().unwrap())?,
        op: parse_comp_op(pairs.next().unwrap())?,
        rhs: parse_comparable(pairs.next().unwrap())?,
    })
}

fn parse_comparable(pair: Pair<Rule>) -> Result<Operand> {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::RelPath | Rule::JsonPath => Ok(Operand::Path(parse_jsonpath(pair.into_inner())?)),
        Rule::Number => Ok(Operand::Value(parse_float(pair).into())),
        Rule::String => Ok(Operand::Value(parse_string(pair).into())),
        Rule::Boolean => Ok(Operand::Value(parse_bool(pair).into())),
        Rule::Null => Ok(Operand::Value(Value::Null)),
        rule => unmatched_rule(rule),
    }
}

fn parse_comp_op(pair: Pair<Rule>) -> Result<CompOp> {
    CompOp::from_str(pair.as_str())
}

fn parse_contain_expr(pair: Pair<Rule>) -> Result<CompExpr> {
    let mut pairs = pair.into_inner();

    Ok(CompExpr {
        lhs: parse_containable(pairs.next().unwrap())?,
        op: CompOp::In,
        rhs: parse_container(pairs.next().unwrap())?,
    })
}

fn parse_containable(pair: Pair<Rule>) -> Result<Operand> {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::RelPath | Rule::JsonPath => Ok(Operand::Path(parse_jsonpath(pair.into_inner())?)),
        Rule::Number => Ok(Operand::Value(parse_float(pair).into())),
        Rule::String => Ok(Operand::Value(parse_string(pair).into())),
        rule => unmatched_rule(rule),
    }
}

fn parse_container(pair: Pair<Rule>) -> Result<Operand> {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::RelPath | Rule::JsonPath => Ok(Operand::Path(parse_jsonpath(pair.into_inner())?)),
        Rule::Array => Ok(Operand::Value(
            serde_json::from_str(pair.as_str()).map_err(Error::new)?,
        )),
        Rule::Object => Ok(Operand::Value(
            serde_json::from_str(pair.as_str()).map_err(Error::new)?,
        )),
        rule => unmatched_rule(rule),
    }
}

fn parse_string(pair: Pair<Rule>) -> String {
    inner(pair).as_str().to_owned()
}

fn parse_quoted_string(pair: Pair<Rule>) -> String {
    parse_string(inner(pair))
}

fn parse_bool(pair: Pair<Rule>) -> bool {
    pair.as_str().parse().unwrap()
}

fn parse_int(pair: Pair<Rule>) -> i64 {
    pair.as_str().parse().unwrap()
}

fn parse_float(pair: Pair<Rule>) -> f64 {
    pair.as_str().parse().unwrap()
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

#[track_caller]
fn unmatched_rule(rule: Rule) -> ! {
    panic!("unmatched rule: {:?}", rule)
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[test]
    fn test_parse_root() {
        let parsed = parse("$").unwrap();
        assert_eq!(parsed, vec![Selector::Root])
    }

    #[test]
    fn test_parse_dot() {
        let parsed = parse("$.foo").unwrap();
        assert_eq!(parsed, vec![Selector::Root, Selector::Key("foo".into())]);

        let parsed = parse("$.*").unwrap();
        assert_eq!(parsed, vec![Selector::Root, Selector::Wildcard])
    }

    #[test]
    fn test_parse_wildcard() {
        let parsed = parse("$.*").unwrap();
        assert_eq!(parsed, vec![Selector::Root, Selector::Wildcard])
    }

    #[test]
    fn test_parse_index() {
        let parsed = parse("$[1]").unwrap();
        assert_eq!(parsed, vec![Selector::Root, Selector::Index(1)]);

        let parsed = parse(r#"$["foo\""]"#).unwrap();
        assert_eq!(
            parsed,
            vec![Selector::Root, Selector::Key(r#"foo\""#.into())]
        );
    }

    #[test]
    fn test_parse_index_wildcard() {
        let parsed = parse("$[*]").unwrap();
        assert_eq!(parsed, vec![Selector::Root, Selector::IndexWildcard])
    }

    #[test]
    fn test_parse_descendant() {
        let parsed = parse("$..[1]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Descendant(Box::new(Selector::Index(1)))
            ]
        );

        let parsed = parse("$..[\"foo\"]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Descendant(Box::new(Selector::Key("foo".into())))
            ]
        );

        let parsed = parse("$..*").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Descendant(Box::new(Selector::Wildcard))
            ]
        );

        let parsed = parse("$..[*]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Descendant(Box::new(Selector::IndexWildcard))
            ]
        );
    }

    #[test]
    fn test_parse_slice() {
        let parsed = parse("$[]").unwrap();
        assert_eq!(
            parsed,
            vec![Selector::Root, Selector::Slice(SliceRange::default())]
        );
        let parsed = parse("$[:]").unwrap();
        assert_eq!(
            parsed,
            vec![Selector::Root, Selector::Slice(SliceRange::default())]
        );
        let parsed = parse("$[::]").unwrap();
        assert_eq!(
            parsed,
            vec![Selector::Root, Selector::Slice(SliceRange::default())]
        );
        let parsed = parse("$[1:]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Slice(SliceRange {
                    start: Some(1),
                    end: None,
                    step: None
                })
            ]
        );
        let parsed = parse("$[1:2]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Slice(SliceRange {
                    start: Some(1),
                    end: Some(2),
                    step: None
                })
            ]
        );
        let parsed = parse("$[:-1]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Slice(SliceRange {
                    start: None,
                    end: Some(-1),
                    step: None
                })
            ]
        );
        let parsed = parse("$[1:2:3]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Slice(SliceRange {
                    start: Some(1),
                    end: Some(2),
                    step: Some(3),
                })
            ]
        );
    }

    #[test]
    fn test_parse_union() {
        let parsed = parse("$[1:2:3,\"foo\",1]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Union(vec![
                    Selector::Slice(SliceRange {
                        start: Some(1),
                        end: Some(2),
                        step: Some(3),
                    }),
                    Selector::Key("foo".into()),
                    Selector::Index(1),
                ])
            ]
        );
    }

    #[test]
    fn test_parse_filter() {
        let parsed = parse("$[?(@ =~ /foo/)]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Regex(RegexExpr {
                    lhs: Operand::Path(vec![Selector::Current]),
                    regex: regex::Regex::new("foo").unwrap()
                }))
            ]
        );

        let parsed = parse("$[?(!(@ =~ /foo/))]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Not(Box::new(FilterExpr::Regex(RegexExpr {
                    lhs: Operand::Path(vec![Selector::Current]),
                    regex: regex::Regex::new("foo").unwrap()
                }))))
            ]
        );

        let parsed = parse("$[?(@ =~ /foo/ && @.bar =~ /qux/)]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::And(vec![
                    FilterExpr::Regex(RegexExpr {
                        lhs: Operand::Path(vec![Selector::Current]),
                        regex: regex::Regex::new("foo").unwrap()
                    }),
                    FilterExpr::Regex(RegexExpr {
                        lhs: Operand::Path(vec![Selector::Current, Selector::Key("bar".into())]),
                        regex: regex::Regex::new("qux").unwrap()
                    }),
                ]))
            ]
        );

        let parsed = parse("$[?(@.foo)]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Exist(vec![
                    Selector::Current,
                    Selector::Key("foo".into())
                ]))
            ]
        );

        let parsed = parse("$[?(@.foo >= 1)]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Comp(CompExpr {
                    lhs: Operand::Path(vec![Selector::Current, Selector::Key("foo".into())]),
                    op: CompOp::GreaterEq,
                    rhs: Operand::Value(1.0.into())
                }))
            ]
        );

        let parsed = parse("$[?(@.foo == 'bar')]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Comp(CompExpr {
                    lhs: Operand::Path(vec![Selector::Current, Selector::Key("foo".into())]),
                    op: CompOp::Eq,
                    rhs: Operand::Value("bar".into())
                }))
            ]
        );

        let parsed = parse("$[?(@.foo in [1, 2])]").unwrap();
        assert_eq!(
            parsed,
            vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Comp(CompExpr {
                    lhs: Operand::Path(vec![Selector::Current, Selector::Key("foo".into())]),
                    op: CompOp::In,
                    rhs: Operand::Value(json!([1, 2]))
                }))
            ]
        );
    }
}
