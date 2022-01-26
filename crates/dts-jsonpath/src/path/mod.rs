mod filter;
mod selector;

use crate::parser::ast::{self, FilterExpr, Selector};
use filter::*;
use selector::*;

pub(crate) use selector::{
    JsonPath, PathPointer, PathPointerMut, PathSelector, PathVisitor, Visitor,
};

pub fn compile(selectors: Vec<Selector>) -> JsonPath {
    JsonPath::Chain(ChainSelector::from_iter(
        selectors.into_iter().map(compile_selector),
    ))
}

fn compile_selector(selector: Selector) -> JsonPath {
    match selector {
        Selector::Root => JsonPath::Root(RootSelector),
        Selector::Current => JsonPath::Current(CurrentSelector),
        Selector::Key(key) => JsonPath::Key(KeySelector::new(key)),
        Selector::Wildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Index(index) => JsonPath::Index(IndexSelector::new(index)),
        Selector::IndexWildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Union(entries) => JsonPath::Union(UnionSelector::new(
            entries.into_iter().map(compile_selector),
        )),
        Selector::Slice(range) => JsonPath::Slice(SliceSelector::new(range.into())),
        Selector::Descendant(selector) => {
            JsonPath::Descendant(DescendantSelector::new(compile_selector(*selector)))
        }
        Selector::Filter(expr) => JsonPath::Filter(FilterSelector::new(compile_filter(expr))),
    }
}

fn compile_filter(expr: FilterExpr) -> Filter {
    match expr {
        FilterExpr::Not(expr) => Filter::Not(Box::new(compile_filter(*expr))),
        FilterExpr::Or(exprs) => Filter::Or(exprs.into_iter().map(compile_filter).collect()),
        FilterExpr::And(exprs) => Filter::And(exprs.into_iter().map(compile_filter).collect()),
        FilterExpr::Exist(path) => Filter::Exist(compile(path)),
        FilterExpr::Regex(expr) => Filter::Regex(RegexFilter::new(expr.lhs.into(), expr.regex)),
        FilterExpr::Comp(expr) => Filter::Comp(CompFilter::new(
            expr.lhs.into(),
            expr.op.into(),
            expr.rhs.into(),
        )),
    }
}

impl From<ast::Operand> for Operand {
    fn from(oper: ast::Operand) -> Self {
        match oper {
            ast::Operand::Value(v) => Operand::Value(v),
            ast::Operand::Path(path) => Operand::Path(compile(path)),
        }
    }
}

impl From<ast::CompOp> for CompOp {
    fn from(op: ast::CompOp) -> Self {
        match op {
            ast::CompOp::Eq => CompOp::Eq,
            ast::CompOp::NotEq => CompOp::NotEq,
            ast::CompOp::LessEq => CompOp::LessEq,
            ast::CompOp::Less => CompOp::Less,
            ast::CompOp::GreaterEq => CompOp::GreaterEq,
            ast::CompOp::Greater => CompOp::Greater,
            ast::CompOp::In => CompOp::In,
        }
    }
}

impl From<ast::SliceRange> for SliceRange {
    fn from(range: ast::SliceRange) -> Self {
        SliceRange::new(range.start, range.end, range.step)
    }
}
