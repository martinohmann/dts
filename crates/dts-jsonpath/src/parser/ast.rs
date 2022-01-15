use dts_json::{Map, Number, Value};
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct JsonPath(pub(super) Vec<JsonPathSelector>);

#[derive(Debug, PartialEq)]
pub struct RelPath(Vec<RelPathSelector>);

#[derive(Debug, PartialEq)]
pub enum Path {
    JsonPath(JsonPath),
    RelPath(RelPath),
}

#[derive(Debug, PartialEq)]
pub enum JsonPathSelector {
    Root,
    Dot(String),
    Wildcard,
    Index(IndexSelector),
    IndexWildcard,
    Union(UnionSelector),
    Slice(Slice),
    Descendant(DescendantSelector),
    Filter(FilterSelector),
}

#[derive(Debug, PartialEq)]
pub enum RelPathSelector {
    Current,
    Dot(String),
    IndexSelector(IndexSelector),
}

#[derive(Debug, PartialEq)]
pub struct UnionSelector(pub(super) Vec<UnionEntry>);

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

#[derive(Debug, PartialEq)]
pub struct Slice {
    pub(super) start: Option<i64>,
    pub(super) end: Option<i64>,
    pub(super) step: Option<i64>,
}

#[derive(Debug, PartialEq)]
pub enum DescendantSelector {
    Key(String),
    Index(IndexSelector),
    IndexWildcard,
    Wildcard,
}

#[derive(Debug, PartialEq)]
pub struct FilterSelector(FilterExpr);

#[derive(Debug, PartialEq)]
pub enum FilterExpr {
    NotExpr,
    LogicalOrExpr(Vec<FilterExpr>),
    LogicalAndExpr(Vec<FilterExpr>),
    ParenExpr(ParenExpr),
    ExistExpr(ExistExpr),
    CompExpr(CompExpr),
    RegexExpr(RegexExpr),
    ContainExpr(ContainExpr),
}

#[derive(Debug, PartialEq)]
pub struct NotExpr(Box<FilterExpr>);

#[derive(Debug, PartialEq)]
pub struct ParenExpr(Box<FilterExpr>);

#[derive(Debug, PartialEq)]
pub struct ExistExpr(Path);

#[derive(Debug, PartialEq)]
pub struct CompExpr {
    lhs: Comparable,
    rhs: Comparable,
    op: CompOp,
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

#[derive(Debug, PartialEq)]
pub enum Comparable {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Path(Path),
}

#[derive(Debug)]
pub struct RegexExpr(Regex);

impl PartialEq for RegexExpr {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string().eq(&other.0.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct ContainExpr {
    containable: Containable,
    container: Container,
}

#[derive(Debug, PartialEq)]
pub enum Containable {
    Number(Number),
    String(String),
    Path(Box<Path>),
}

#[derive(Debug, PartialEq)]
pub enum Container {
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Path(Box<Path>),
}
