use crate::Error;
use dts_json::{Map, Value};
use regex::Regex;
use std::str::FromStr;

pub type JsonPath = Vec<Selector>;

#[derive(Debug, PartialEq)]
pub enum Selector {
    Root,
    Current,
    Key(String),
    Wildcard,
    Index(i64),
    IndexWildcard,
    Union(JsonPath),
    Slice(SliceSelector),
    Descendant(Box<Selector>),
    Filter(FilterExpr),
}

#[derive(Debug, PartialEq, Default)]
pub struct SliceSelector {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<i64>,
}

#[derive(Debug, PartialEq)]
pub enum FilterExpr {
    Not(Box<FilterExpr>),
    Or(Vec<FilterExpr>),
    And(Vec<FilterExpr>),
    Exist(JsonPath),
    Comp(CompExpr),
    Regex(RegexExpr),
    Contain(ContainExpr),
}

#[derive(Debug, PartialEq)]
pub struct CompExpr {
    pub lhs: Comparable,
    pub op: CompOp,
    pub rhs: Comparable,
}

#[derive(Debug, PartialEq)]
pub enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
}

impl FromStr for CompOp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(CompOp::Eq),
            "!=" => Ok(CompOp::NotEq),
            "<=" => Ok(CompOp::LessEq),
            "<" => Ok(CompOp::Less),
            ">=" => Ok(CompOp::GreaterEq),
            ">" => Ok(CompOp::Greater),
            other => Err(Error::new(format!(
                "not a comparision operation: {}",
                other
            ))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Comparable {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Path(JsonPath),
}

#[derive(Debug)]
pub struct RegexExpr {
    pub matchable: RegexMatchable,
    pub regex: Regex,
}

#[derive(Debug, PartialEq)]
pub enum RegexMatchable {
    String(String),
    Path(JsonPath),
}

impl PartialEq for RegexExpr {
    fn eq(&self, other: &Self) -> bool {
        match (&self.matchable, &other.matchable) {
            (RegexMatchable::String(s1), RegexMatchable::String(s2)) => {
                s1 == s2 && self.regex.to_string() == other.regex.to_string()
            }
            (RegexMatchable::Path(p1), RegexMatchable::Path(p2)) => {
                p1 == p2 && self.regex.to_string() == other.regex.to_string()
            }
            (_, _) => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ContainExpr {
    pub containable: Containable,
    pub container: Container,
}

#[derive(Debug, PartialEq)]
pub enum Containable {
    Number(f64),
    String(String),
    Path(JsonPath),
}

#[derive(Debug, PartialEq)]
pub enum Container {
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Path(JsonPath),
}
