use dts_json::{Map, Number, Value};
use regex::Regex;

struct Path(Vec<Selector>);

enum Selector {
    Root(RootSelector),
    Current(CurrentSelector),
    Dot(DotSelector),
    Wildcard,
    Index(IndexSelector),
    IndexWildcard,
    Union(UnionSelector),
    Slice(SliceSelector),
    Descendant,
    Filter(Expr),
}

struct RootSelector;

struct CurrentSelector(Box<Path>);

struct DotSelector(String);

struct UnionSelector(Vec<UnionEntry>);

struct SliceSelector(Option<SliceIndex>);

enum IndexSelector {
    Index(i64),
    Key(String),
}

enum UnionEntry {
    Key(String),
    Index(i64),
    SliceIndex(SliceIndex),
}

struct SliceIndex {
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
}

enum DescendantSelector {
    Key(String),
    Index(i64),
    IndexWildcard,
    Wildcard,
}

struct FilterSelector(Expr);

enum Expr {
    NotExpr,
    LogicalOrExpr(Vec<Expr>),
    LogicalAndExpr(Vec<Expr>),
    ParenExpr(ParenExpr),
    ExistExpr(ExistExpr),
    CompExpr(CompExpr),
    RegexExpr(RegexExpr),
    ContainExpr(ContainExpr),
}

struct NotExpr(Box<Expr>);

struct ParenExpr(Box<Expr>);

struct ExistExpr(Box<Path>);

struct CompExpr {
    lhs: Comparable,
    rhs: Comparable,
    op: CompOp,
}

enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
}

enum Comparable {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Path(Box<Path>),
}

struct RegexExpr(Regex);

struct ContainExpr {
    containable: Containable,
    container: Container,
}

enum Containable {
    Number(Number),
    String(String),
    Path(Box<Path>),
}

enum Container {
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Path(Box<Path>),
}
