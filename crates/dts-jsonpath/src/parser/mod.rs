#![allow(unused_imports)]

mod ast;

use ast::*;

use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParserTrait;
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/grammar/jsonpath.pest"]
struct JsonPathParser;

pub fn parse(input: &str) -> Result<JsonPath> {
    let pairs = JsonPathParser::parse(Rule::Root, input)?;

    parse_jsonpath(pairs)
}

fn parse_jsonpath(pairs: Pairs<Rule>) -> Result<JsonPath> {
    Ok(JsonPath(
        pairs
            .take_while(|pair| pair.as_rule() != Rule::EOI)
            .map(parse_selector)
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn parse_selector(pair: Pair<Rule>) -> Result<Selector> {
    match pair.as_rule() {
        Rule::RootSelector => Ok(Selector::Root),
        Rule::CurrentSelector => Ok(Selector::Current),
        Rule::DotSelector => Ok(Selector::Key(parse_dot(pair))),
        Rule::DotWildSelector => Ok(Selector::Wildcard),
        Rule::IndexSelector => Ok(Selector::Index(parse_index(pair))),
        Rule::IndexWildSelector => Ok(Selector::IndexWildcard),
        Rule::UnionSelector => Ok(Selector::Union(parse_union(pair))),
        Rule::SliceSelector => Ok(Selector::Slice(parse_slice(inner(pair)))),
        Rule::DescendantSelector => Ok(Selector::Descendant(parse_descendant(pair))),
        Rule::FilterSelector => Ok(Selector::Filter(parse_filter_expr(inner(pair))?)),
        rule => unreachable_rule(rule),
    }
}

fn parse_dot(pair: Pair<Rule>) -> String {
    inner(pair).as_str().to_owned()
}

fn parse_quoted_string(pair: Pair<Rule>) -> String {
    parse_dot(inner(pair))
}

fn parse_index(pair: Pair<Rule>) -> IndexSelector {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::ElementIndex => IndexSelector::Index(pair.as_str().parse().unwrap()),
        Rule::QuotedMemberName => IndexSelector::Key(parse_quoted_string(pair)),
        rule => unreachable_rule(rule),
    }
}

fn parse_descendant(pair: Pair<Rule>) -> Descendant {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::DotMemberName => Descendant::Key(parse_dot(pair)),
        Rule::IndexSelector => Descendant::Index(parse_index(pair)),
        Rule::IndexWildSelector => Descendant::IndexWildcard,
        Rule::Wildcard => Descendant::Wildcard,
        rule => unreachable_rule(rule),
    }
}

fn parse_slice(pair: Pair<Rule>) -> Slice {
    let indices = pair
        .into_inner()
        .map(|pair| pair.into_inner().next().unwrap())
        .map(parse_slice_index)
        .collect::<Vec<i64>>();

    let slice = Slice::new();

    match &indices.len() {
        0 => slice,
        1 => slice.with_start(indices[0]),
        2 => slice.with_start(indices[0]).with_end(indices[1]),
        3 => slice
            .with_start(indices[0])
            .with_end(indices[1])
            .with_step(indices[2]),
        _ => unreachable!(),
    }
}

fn parse_slice_index(pair: Pair<Rule>) -> i64 {
    pair.as_str().parse().unwrap()
}

fn parse_union(pair: Pair<Rule>) -> Vec<UnionEntry> {
    pair.into_inner()
        .map(|pair| {
            let pair = pair.into_inner().next().unwrap();

            match pair.as_rule() {
                Rule::ElementIndex => UnionEntry::Index(pair.as_str().parse().unwrap()),
                Rule::QuotedMemberName => UnionEntry::Key(parse_quoted_string(pair)),
                Rule::SliceIndex => UnionEntry::Slice(parse_slice(pair)),
                rule => unreachable_rule(rule),
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
        Rule::ContainExpr => FilterExpr::Contain(parse_contain_expr(pair)?),
        rule => panic!("unexpected filter expr: {:?}", rule),
    };

    // Unwrap single expr or/and exprs.
    let expr = match expr {
        FilterExpr::Or(mut es) | FilterExpr::And(mut es) if es.len() == 1 => es.swap_remove(0),
        expr => expr,
    };

    Ok(expr)
}

fn parse_filter_exprs(pair: Pair<Rule>) -> Result<Vec<FilterExpr>> {
    pair.into_inner()
        .map(parse_filter_expr)
        .collect::<Result<Vec<_>>>()
}

fn parse_regex_expr(pair: Pair<Rule>) -> Result<Regex> {
    let mut pairs = pair.into_inner();

    let operand = pairs.next().unwrap();
    let re = parse_regex(inner(pairs.next().unwrap()))?;

    match operand.as_rule() {
        Rule::RelPath | Rule::JsonPath => {
            Ok(Regex::Path(parse_jsonpath(operand.into_inner())?, re))
        }
        Rule::String => Ok(Regex::String(parse_quoted_string(operand), re)),
        rule => unreachable_rule(rule),
    }
}

fn parse_regex(pair: Pair<Rule>) -> Result<regex::Regex> {
    // @TODO(mohmann): add custom regex variant to `Error`
    regex::Regex::new(pair.as_str()).map_err(Error::new)
}

fn parse_comp_expr(pair: Pair<Rule>) -> Result<CompExpr> {
    let mut pairs = pair.into_inner();

    let lhs = parse_comparable(pairs.next().unwrap())?;
    let op = parse_comp_op(pairs.next().unwrap())?;
    let rhs = parse_comparable(pairs.next().unwrap())?;

    Ok(CompExpr { lhs, op, rhs })
}

fn parse_comparable(_pair: Pair<Rule>) -> Result<Comparable> {
    unimplemented!()
}

fn parse_comp_op(pair: Pair<Rule>) -> Result<CompOp> {
    CompOp::from_str(pair.as_str())
}

fn parse_contain_expr(pair: Pair<Rule>) -> Result<ContainExpr> {
    let mut pairs = pair.into_inner();

    let containable = parse_containable(pairs.next().unwrap())?;
    let container = parse_container(pairs.next().unwrap())?;

    Ok(ContainExpr {
        containable,
        container,
    })
}

fn parse_containable(_pair: Pair<Rule>) -> Result<Containable> {
    unimplemented!()
}

fn parse_container(_pair: Pair<Rule>) -> Result<Container> {
    unimplemented!()
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

#[track_caller]
fn unreachable_rule(rule: Rule) -> ! {
    panic!("unreachable rule: {:?}", rule)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_root() {
        let parsed = parse("$").unwrap();
        assert_eq!(parsed, JsonPath(vec![Selector::Root]))
    }

    #[test]
    fn test_parse_dot() {
        let parsed = parse("$.foo").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![Selector::Root, Selector::Key("foo".into())])
        );

        let parsed = parse("$.*").unwrap();
        assert_eq!(parsed, JsonPath(vec![Selector::Root, Selector::Wildcard]))
    }

    #[test]
    fn test_parse_wildcard() {
        let parsed = parse("$.*").unwrap();
        assert_eq!(parsed, JsonPath(vec![Selector::Root, Selector::Wildcard]))
    }

    #[test]
    fn test_parse_index() {
        let parsed = parse("$[1]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Index(IndexSelector::Index(1))
            ])
        );

        let parsed = parse(r#"$["foo\""]"#).unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Index(IndexSelector::Key(r#"foo\""#.into()))
            ])
        );
    }

    #[test]
    fn test_parse_index_wildcard() {
        let parsed = parse("$[*]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![Selector::Root, Selector::IndexWildcard])
        )
    }

    #[test]
    fn test_parse_descendant() {
        let parsed = parse("$..[1]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Descendant(Descendant::Index(IndexSelector::Index(1)))
            ])
        );

        let parsed = parse("$..[\"foo\"]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Descendant(Descendant::Index(IndexSelector::Key("foo".into())))
            ])
        );

        let parsed = parse("$..*").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Descendant(Descendant::Wildcard)
            ])
        );

        let parsed = parse("$..[*]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Descendant(Descendant::IndexWildcard)
            ])
        );
    }

    #[test]
    fn test_parse_slice() {
        let parsed = parse("$[1:2:3]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Slice(Slice {
                    start: Some(1),
                    end: Some(2),
                    step: Some(3),
                })
            ])
        );
    }

    #[test]
    fn test_parse_union() {
        let parsed = parse("$[1:2:3,\"foo\",1]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Union(vec![
                    UnionEntry::Slice(Slice {
                        start: Some(1),
                        end: Some(2),
                        step: Some(3),
                    }),
                    UnionEntry::Key("foo".into()),
                    UnionEntry::Index(1),
                ])
            ])
        );
    }

    #[test]
    fn test_parse_filter() {
        let parsed = parse("$[?(@ =~ /foo/)]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Regex(Regex::Path(
                    JsonPath(vec![Selector::Current]),
                    regex::Regex::new("foo").unwrap()
                )))
            ])
        );

        let parsed = parse("$[?(!(@ =~ /foo/))]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Not(Box::new(FilterExpr::Regex(Regex::Path(
                    JsonPath(vec![Selector::Current]),
                    regex::Regex::new("foo").unwrap()
                )))))
            ])
        );

        let parsed = parse("$[?(@ =~ /foo/ && @.bar =~ /qux/)]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Filter(FilterExpr::And(vec![
                    FilterExpr::Regex(Regex::Path(
                        JsonPath(vec![Selector::Current]),
                        regex::Regex::new("foo").unwrap()
                    )),
                    FilterExpr::Regex(Regex::Path(
                        JsonPath(vec![Selector::Current, Selector::Key("bar".into())]),
                        regex::Regex::new("qux").unwrap()
                    ))
                ]))
            ])
        );

        let parsed = parse("$[?(@.foo)]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                Selector::Root,
                Selector::Filter(FilterExpr::Exist(JsonPath(vec![
                    Selector::Current,
                    Selector::Key("foo".into())
                ])))
            ])
        );
    }
}
