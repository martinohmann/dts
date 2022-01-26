mod filter;
mod selector;

use crate::parser::ast::{self, FilterExpr, Selector};
use dts_json::Value;
use filter::*;
use selector::*;

pub(crate) use selector::{JsonPath, PathSelector, Visitor};

pub fn compile<'a>(selectors: &'a [Selector], root: &'a Value) -> JsonPath<'a> {
    JsonPath::Chain(ChainSelector::from_iter(
        selectors.into_iter().map(|sel| compile_selector(sel, root)),
    ))
}

fn compile_selector<'a>(selector: &'a Selector, root: &'a Value) -> JsonPath<'a> {
    match selector {
        Selector::Root => JsonPath::Root(RootSelector::new(root)),
        Selector::Current => JsonPath::Current(CurrentSelector),
        Selector::Key(key) => JsonPath::Key(KeySelector::new(key.clone())),
        Selector::Wildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Index(index) => JsonPath::Index(IndexSelector::new(*index)),
        Selector::IndexWildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Union(entries) => JsonPath::Union(UnionSelector::new(
            entries.into_iter().map(|sel| compile_selector(sel, root)),
        )),
        Selector::Slice(range) => JsonPath::Slice(SliceSelector::new(range.into())),
        Selector::Descendant(selector) => {
            JsonPath::Descendant(DescendantSelector::new(compile_selector(selector, root)))
        }
        Selector::Filter(expr) => JsonPath::Filter(FilterSelector::new(compile_filter(expr, root))),
    }
}

fn compile_filter<'a>(expr: &'a FilterExpr, root: &'a Value) -> Filter<'a> {
    match expr {
        FilterExpr::Not(expr) => Filter::Not(Box::new(compile_filter(expr, root))),
        FilterExpr::Or(exprs) => Filter::Or(
            exprs
                .into_iter()
                .map(|expr| compile_filter(expr, root))
                .collect(),
        ),
        FilterExpr::And(exprs) => Filter::And(
            exprs
                .into_iter()
                .map(|expr| compile_filter(expr, root))
                .collect(),
        ),
        FilterExpr::Exist(path) => Filter::Exist(compile(path, root)),
        FilterExpr::Regex(expr) => Filter::Regex(RegexFilter::new(
            compile_operand(&expr.lhs, root),
            expr.regex.clone(),
        )),
        FilterExpr::Comp(expr) => Filter::Comp(CompFilter::new(
            compile_operand(&expr.lhs, root),
            expr.op.into(),
            compile_operand(&expr.rhs, root),
        )),
    }
}

fn compile_operand<'a>(oper: &'a ast::Operand, root: &'a Value) -> JsonPath<'a> {
    match oper {
        ast::Operand::Value(v) => JsonPath::Root(RootSelector::new(v)),
        ast::Operand::Path(path) => compile(path, root),
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

impl From<&ast::SliceRange> for SliceRange {
    fn from(range: &ast::SliceRange) -> Self {
        SliceRange::new(range.start, range.end, range.step)
    }
}
