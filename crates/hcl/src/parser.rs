use crate::{ast::*, Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParseTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/hcl.pest"]
pub(crate) struct HclParser;

pub fn parse(s: &str) -> Result<Body<'_>, Error> {
    let body = HclParser::parse(Rule::hcl, s)
        .map_err(|e| Error::ParseError(e.to_string()))?
        .filter_map(parse_structure)
        .collect();

    Ok(body)
}

fn parse_structure(pair: Pair<Rule>) -> Option<Structure> {
    match pair.as_rule() {
        Rule::attribute => Some(parse_attribute(pair.into_inner())),
        Rule::block => Some(parse_block(pair.into_inner())),
        Rule::EOI => None,
        _ => unreachable!(),
    }
}

fn parse_attribute(mut pairs: Pairs<Rule>) -> Structure {
    let ident = parse_identifier(pairs.next().unwrap());
    let expr = parse_expression(inner(pairs.next().unwrap()));
    Structure::Attribute(ident, expr)
}

fn parse_block(mut pairs: Pairs<Rule>) -> Structure {
    let ident = parse_block_identifier(pairs.next().unwrap().into_inner());
    let body = parse_block_body(pairs.next().unwrap().into_inner());
    Structure::Block(ident, Box::new(body))
}

fn parse_block_identifier(pairs: Pairs<Rule>) -> Vec<&str> {
    pairs.map(parse_identifier).collect()
}

fn parse_block_body(pairs: Pairs<Rule>) -> Body {
    pairs.filter_map(parse_structure).collect()
}

fn parse_identifier(ident: Pair<Rule>) -> &str {
    match ident.as_rule() {
        Rule::identifier | Rule::string => ident.as_str(),
        _ => unreachable!(),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::value => Expression::Value(parse_value(inner(pair))),
        // For now, do not distinguish between any other expressions just map the as expose them
        // as RawExpr.
        _ => Expression::RawExpr(pair.as_str()),
    }
}

fn parse_value(pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::null_lit => Value::Null,
        Rule::boolean_lit => Value::Bool(pair.as_str().parse().unwrap()),
        Rule::numeric_lit => Value::Number(parse_number(inner(pair))),
        Rule::string => Value::String(pair.as_str()),
        Rule::heredoc => Value::String(parse_heredoc(pair.into_inner())),
        Rule::object => Value::Object(parse_object(pair.into_inner())),
        Rule::tuple => Value::Tuple(parse_tuple(pair.into_inner())),
        _ => unreachable!(),
    }
}

fn parse_heredoc(mut pairs: Pairs<Rule>) -> &str {
    // The first pair is the heredoc identifier, e.g. `HEREDOC`, the second one is the template
    // that we are interested in.
    pairs.nth(1).unwrap().as_str()
}

fn parse_object(pairs: Pairs<Rule>) -> Vec<ObjectItem> {
    pairs
        .map(|pair| parse_object_item(pair.into_inner()))
        .collect()
}

fn parse_object_item(mut pairs: Pairs<Rule>) -> ObjectItem {
    let key = parse_object_key(pairs.next().unwrap());
    let expr = parse_expression(inner(pairs.next().unwrap()));
    ObjectItem(key, expr)
}

fn parse_object_key(pair: Pair<Rule>) -> ObjectKey {
    match pair.as_rule() {
        Rule::identifier => ObjectKey::Identifier(pair.as_str()),
        Rule::expression => ObjectKey::Expression(parse_expression(inner(pair))),
        _ => unreachable!(),
    }
}

fn parse_tuple(pairs: Pairs<Rule>) -> Vec<Expression> {
    pairs.map(|pair| parse_expression(inner(pair))).collect()
}

fn parse_number(pair: Pair<Rule>) -> Number {
    match pair.as_rule() {
        Rule::float => Number::Float(pair.as_str().parse().unwrap()),
        Rule::int => Number::Int(pair.as_str().parse().unwrap()),
        _ => unreachable!(),
    }
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}
