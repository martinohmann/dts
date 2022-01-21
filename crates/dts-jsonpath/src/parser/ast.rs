use crate::Error;
use dts_json::Value;
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
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Value(Value),
    Path(JsonPath),
}

#[derive(Debug, PartialEq)]
pub struct CompExpr {
    pub lhs: Operand,
    pub op: CompOp,
    pub rhs: Operand,
}

#[derive(Debug, PartialEq)]
pub enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    In,
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
            "in" => Ok(CompOp::In),
            other => Err(Error::new(format!(
                "not a comparision operation: {}",
                other
            ))),
        }
    }
}

#[derive(Debug)]
pub struct RegexExpr {
    pub lhs: Operand,
    pub regex: Regex,
}

impl PartialEq for RegexExpr {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.regex.to_string() == other.regex.to_string()
    }
}
