use crate::Error;
use dts_json::{Map, Number, Value};
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct JsonPath(pub(super) Vec<Selector>);

#[derive(Debug, PartialEq)]
pub enum Selector {
    Root,
    Current,
    Key(String),
    Wildcard,
    Index(IndexSelector),
    IndexWildcard,
    Union(Vec<UnionEntry>),
    Slice(Slice),
    Descendant(Descendant),
    Filter(FilterExpr),
}

#[derive(Debug, PartialEq)]
pub enum IndexSelector {
    Index(i64),
    Key(String),
}

#[derive(Debug, PartialEq)]
pub enum UnionEntry {
    Key(String),
    Index(i64),
    Slice(Slice),
}

#[derive(Debug, PartialEq, Default)]
pub struct Slice {
    pub(super) start: Option<i64>,
    pub(super) end: Option<i64>,
    pub(super) step: Option<i64>,
}

impl Slice {
    pub fn new() -> Self {
        Slice::default()
    }

    pub fn with_start(mut self, start: i64) -> Self {
        self.start = Some(start);
        self
    }

    pub fn with_end(mut self, end: i64) -> Self {
        self.end = Some(end);
        self
    }

    pub fn with_step(mut self, step: i64) -> Self {
        self.step = Some(step);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum Descendant {
    Key(String),
    Index(IndexSelector),
    IndexWildcard,
    Wildcard,
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
    pub(super) lhs: Comparable,
    pub(super) op: CompOp,
    pub(super) rhs: Comparable,
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
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Path(JsonPath),
}

#[derive(Debug)]
pub enum RegexExpr {
    String(String, Regex),
    Path(JsonPath, Regex),
}

impl PartialEq for RegexExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RegexExpr::String(s1, re1), RegexExpr::String(s2, re2)) => {
                s1 == s2 && re1.to_string() == re2.to_string()
            }
            (RegexExpr::Path(p1, re1), RegexExpr::Path(p2, re2)) => {
                p1 == p2 && re1.to_string() == re2.to_string()
            }
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ContainExpr {
    pub(super) containable: Containable,
    pub(super) container: Container,
}

#[derive(Debug, PartialEq)]
pub enum Containable {
    Number(Number),
    String(String),
    Path(JsonPath),
}

#[derive(Debug, PartialEq)]
pub enum Container {
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Path(JsonPath),
}
