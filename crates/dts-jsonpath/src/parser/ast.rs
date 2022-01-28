use crate::path::CompOp;
use dts_json::Value;
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
    Root,
    Current,
    Key(String),
    Wildcard,
    Index(i64),
    IndexWildcard,
    Union(Vec<Selector>),
    Slice(SliceRange),
    Descendant(Box<Selector>),
    Filter(FilterExpr),
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SliceRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<i64>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExpr {
    Not(Box<FilterExpr>),
    Or(Vec<FilterExpr>),
    And(Vec<FilterExpr>),
    Exist(Vec<Selector>),
    Comp(CompExpr),
    Regex(RegexExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Value(Value),
    Path(Vec<Selector>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompExpr {
    pub lhs: Operand,
    pub op: CompOp,
    pub rhs: Operand,
}

#[derive(Debug, Clone)]
pub struct RegexExpr {
    pub lhs: Operand,
    pub regex: Regex,
}

impl PartialEq for RegexExpr {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.regex.to_string() == other.regex.to_string()
    }
}
