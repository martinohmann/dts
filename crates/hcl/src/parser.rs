use crate::{ast::*, Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParseTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/hcl.pest"]
pub(crate) struct HclParser;

pub fn parse<'a>(s: &'a str) -> Result<Body<'a>, Error> {
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
        Rule::expr_term => Expression::ExprTerm(parse_expr_term(inner(pair))),
        Rule::conditional => Expression::Conditional(pair.as_str()),
        Rule::operation => Expression::Operation(pair.as_str()),
        _ => unreachable!(),
    }
}

fn parse_expr_term(pair: Pair<Rule>) -> ExprTerm {
    match pair.as_rule() {
        Rule::literal_value => ExprTerm::LiteralValue(parse_literal_value(inner(pair))),
        Rule::collection_value => ExprTerm::CollectionValue(parse_collection_value(inner(pair))),
        Rule::template_expr => ExprTerm::TemplateExpr(parse_template_expr(inner(pair))),
        // For now, do not distinguish between any other expressions just map the as expose them
        // as RawExpr.
        _ => ExprTerm::RawExpr(pair.as_str()),
    }
}

fn parse_template_expr(pair: Pair<Rule>) -> &str {
    match pair.as_rule() {
        Rule::string => pair.as_str(),
        Rule::template => pair.as_str(),
        _ => unreachable!(),
    }
}

fn parse_literal_value(pair: Pair<Rule>) -> LiteralValue {
    match pair.as_rule() {
        Rule::null_lit => LiteralValue::Null,
        Rule::boolean_lit => LiteralValue::Bool(pair.as_str().parse().unwrap()),
        Rule::numeric_lit => LiteralValue::Number(parse_number(inner(pair))),
        Rule::string => LiteralValue::String(pair.as_str()),
        _ => unreachable!(),
    }
}

fn parse_collection_value(pair: Pair<Rule>) -> CollectionValue {
    match pair.as_rule() {
        Rule::object => CollectionValue::Object(parse_object(pair.into_inner())),
        Rule::tuple => CollectionValue::Tuple(parse_tuple(pair.into_inner())),
        _ => unreachable!(),
    }
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
