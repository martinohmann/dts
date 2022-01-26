mod filter;
mod selector;

use crate::parser::ast::{self, FilterExpr, Operand, Selector};
use dts_json::Value;
use filter::*;
use selector::*;

pub(crate) use selector::{JsonPath, PathSelector, Visitor};

pub fn compile<'a>(selectors: &'a [Selector], root: &'a Value) -> JsonPath<'a> {
    JsonPath::Chain(ChainSelector::from_iter(compile_selectors(selectors, root)))
}

fn compile_selectors<'a>(selectors: &'a [Selector], root: &'a Value) -> Vec<JsonPath<'a>> {
    selectors
        .iter()
        .map(|selector| compile_selector(selector, root))
        .collect()
}

fn compile_selector<'a>(selector: &'a Selector, root: &'a Value) -> JsonPath<'a> {
    match selector {
        Selector::Root => JsonPath::Root(RootSelector::new(root)),
        Selector::Current => JsonPath::Current(CurrentSelector),
        Selector::Key(key) => JsonPath::Key(KeySelector::new(key.clone())),
        Selector::Wildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Index(index) => JsonPath::Index(IndexSelector::new(*index)),
        Selector::IndexWildcard => JsonPath::Wildcard(WildcardSelector),
        Selector::Union(entries) => {
            JsonPath::Union(UnionSelector::from_iter(compile_selectors(entries, root)))
        }
        Selector::Slice(range) => JsonPath::Slice(SliceSelector::new(SliceRange::new(
            range.start,
            range.end,
            range.step,
        ))),
        Selector::Descendant(selector) => {
            JsonPath::Descendant(DescendantSelector::new(compile_selector(selector, root)))
        }
        Selector::Filter(expr) => JsonPath::Filter(FilterSelector::new(compile_filter(expr, root))),
    }
}

fn compile_filter<'a>(expr: &'a FilterExpr, root: &'a Value) -> Filter<'a> {
    match expr {
        FilterExpr::Not(expr) => Filter::Not(Box::new(compile_filter(expr, root))),
        FilterExpr::Or(exprs) => Filter::Or(compile_filter_exprs(exprs, root)),
        FilterExpr::And(exprs) => Filter::And(compile_filter_exprs(exprs, root)),
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

fn compile_filter_exprs<'a>(exprs: &'a [FilterExpr], root: &'a Value) -> Vec<Filter<'a>> {
    exprs
        .iter()
        .map(|expr| compile_filter(expr, root))
        .collect()
}

fn compile_operand<'a>(operand: &'a Operand, root: &'a Value) -> JsonPath<'a> {
    match operand {
        Operand::Value(v) => JsonPath::Root(RootSelector::new(v)),
        Operand::Path(path) => compile(path, root),
    }
}

#[derive(Clone)]
pub enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    In,
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

#[derive(Clone)]
pub struct Index {
    index: i64,
}

impl Index {
    pub(crate) fn new(index: i64) -> Self {
        Index { index }
    }

    pub(crate) fn get(&self, len: i64) -> Option<usize> {
        let index = if self.index < 0 {
            len + self.index
        } else {
            self.index
        };

        if index < 0 || index >= len {
            None
        } else {
            Some(index as usize)
        }
    }
}

#[derive(Default, Clone)]
pub struct SliceRange {
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
}

impl SliceRange {
    pub(crate) fn new(start: Option<i64>, end: Option<i64>, step: Option<i64>) -> Self {
        SliceRange { start, end, step }
    }

    pub(crate) fn start(&self, len: i64) -> i64 {
        match self.start {
            Some(start) => start,
            None => {
                if self.step() >= 0 {
                    0
                } else {
                    len - 1
                }
            }
        }
    }

    pub(crate) fn end(&self, len: i64) -> i64 {
        match self.end {
            Some(end) => end,
            None => {
                if self.step() >= 0 {
                    len
                } else {
                    (-len) - 1
                }
            }
        }
    }

    pub(crate) fn step(&self) -> i64 {
        self.step.unwrap_or(1)
    }

    pub(crate) fn bounds(&self, len: i64) -> (i64, i64) {
        fn normalize(i: i64, len: i64) -> i64 {
            if i >= 0 {
                i
            } else {
                len + i
            }
        }

        let step = self.step();
        let start = normalize(self.start(len), len);
        let end = normalize(self.end(len), len);

        let (lower, upper) = if step >= 0 {
            (start.max(0).min(len), end.max(0).min(len))
        } else {
            (end.max(-1).min(len - 1), start.max(-1).min(len - 1))
        };

        (lower, upper)
    }
}
