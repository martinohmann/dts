#![allow(unused_imports)]

mod ast;

use ast::*;

use crate::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParserTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/jsonpath.pest"]
struct JsonPathParser;

pub fn parse(input: &str) -> Result<JsonPath> {
    let path = JsonPathParser::parse(Rule::Root, input)?
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| match pair.as_rule() {
            Rule::RootSelector => Ok(JsonPathSelector::Root),
            Rule::DotSelector => Ok(JsonPathSelector::Dot(parse_dot_member_name(pair))),
            Rule::DotWildSelector => Ok(JsonPathSelector::Wildcard),
            Rule::IndexSelector => Ok(JsonPathSelector::Index(parse_index_selector(pair))),
            Rule::IndexWildSelector => Ok(JsonPathSelector::IndexWildcard),
            Rule::UnionSelector => Ok(JsonPathSelector::Union(parse_union_selector(pair))),
            Rule::SliceSelector => Ok(JsonPathSelector::Slice(parse_slice(
                pair.into_inner().next().unwrap(),
            ))),
            Rule::DescendantSelector => Ok(JsonPathSelector::Descendant(
                parse_descendant_selector(pair),
            )),
            Rule::FilterSelector => unimplemented!(),
            _ => unreachable!(),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(JsonPath(path))
}

fn parse_dot_member_name(pair: Pair<Rule>) -> String {
    pair.into_inner().next().unwrap().as_str().to_owned()
}

fn parse_quoted_member_name(pair: Pair<Rule>) -> String {
    parse_dot_member_name(pair.into_inner().next().unwrap())
}

fn parse_index_selector(pair: Pair<Rule>) -> IndexSelector {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::ElementIndex => IndexSelector::Index(pair.as_str().parse().unwrap()),
        Rule::QuotedMemberName => IndexSelector::Key(parse_quoted_member_name(pair)),
        _ => unreachable!(),
    }
}

fn parse_descendant_selector(pair: Pair<Rule>) -> DescendantSelector {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::DotMemberName => DescendantSelector::Key(parse_dot_member_name(pair)),
        Rule::IndexSelector => DescendantSelector::Index(parse_index_selector(pair)),
        Rule::IndexWildSelector => DescendantSelector::IndexWildcard,
        Rule::Wildcard => DescendantSelector::Wildcard,
        _ => unreachable!(),
    }
}

fn parse_slice(pair: Pair<Rule>) -> Slice {
    let indices = pair
        .into_inner()
        .map(|pair| pair.into_inner().next().unwrap())
        .map(parse_slice_index)
        .collect::<Vec<i64>>();

    match &indices.len() {
        0 => Slice {
            start: None,
            end: None,
            step: None,
        },
        1 => Slice {
            start: Some(indices[0]),
            end: None,
            step: None,
        },
        2 => Slice {
            start: Some(indices[0]),
            end: Some(indices[1]),
            step: None,
        },
        3 => Slice {
            start: Some(indices[0]),
            end: Some(indices[1]),
            step: Some(indices[2]),
        },
        _ => unreachable!(),
    }
}

fn parse_slice_index(pair: Pair<Rule>) -> i64 {
    pair.as_str().parse().unwrap()
}

fn parse_union_selector(pair: Pair<Rule>) -> UnionSelector {
    UnionSelector(pair.into_inner().map(parse_union_entry).collect())
}

fn parse_union_entry(pair: Pair<Rule>) -> UnionEntry {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::ElementIndex => UnionEntry::Index(pair.as_str().parse().unwrap()),
        Rule::QuotedMemberName => UnionEntry::Key(parse_quoted_member_name(pair)),
        Rule::SliceIndex => UnionEntry::Slice(parse_slice(pair)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_root() {
        let parsed = parse("$").unwrap();
        assert_eq!(parsed, JsonPath(vec![JsonPathSelector::Root]))
    }

    #[test]
    fn test_parse_dot() {
        let parsed = parse("$.foo").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Dot("foo".into())
            ])
        );

        let parsed = parse("$.*").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![JsonPathSelector::Root, JsonPathSelector::Wildcard])
        )
    }

    #[test]
    fn test_parse_wildcard() {
        let parsed = parse("$.*").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![JsonPathSelector::Root, JsonPathSelector::Wildcard])
        )
    }

    #[test]
    fn test_parse_index() {
        let parsed = parse("$[1]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Index(IndexSelector::Index(1))
            ])
        );

        let parsed = parse(r#"$["foo\""]"#).unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Index(IndexSelector::Key(r#"foo\""#.into()))
            ])
        );
    }

    #[test]
    fn test_parse_index_wildcard() {
        let parsed = parse("$[*]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::IndexWildcard
            ])
        )
    }

    #[test]
    fn test_parse_descendant() {
        let parsed = parse("$..[1]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Descendant(DescendantSelector::Index(IndexSelector::Index(1)))
            ])
        );

        let parsed = parse("$..[\"foo\"]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Descendant(DescendantSelector::Index(IndexSelector::Key(
                    "foo".into()
                )))
            ])
        );

        let parsed = parse("$..*").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Descendant(DescendantSelector::Wildcard)
            ])
        );

        let parsed = parse("$..[*]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Descendant(DescendantSelector::IndexWildcard)
            ])
        );
    }

    #[test]
    fn test_parse_slice() {
        let parsed = parse("$[1:2:3]").unwrap();
        assert_eq!(
            parsed,
            JsonPath(vec![
                JsonPathSelector::Root,
                JsonPathSelector::Slice(Slice {
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
                JsonPathSelector::Root,
                JsonPathSelector::Union(UnionSelector(vec![
                    UnionEntry::Slice(Slice {
                        start: Some(1),
                        end: Some(2),
                        step: Some(3),
                    }),
                    UnionEntry::Key("foo".into()),
                    UnionEntry::Index(1),
                ]))
            ])
        );
    }
}
