mod filter;
mod selector;

use crate::parser::ast;
use dts_json::Value;
use filter::*;
use selector::*;

pub use selector::{Select, Selector, Visit, Visitor};

pub fn compile<'a>(selectors: &'a [ast::Selector], root: &'a Value) -> Path<'a> {
    compile_selectors(selectors, root).collect()
}

fn compile_selectors<'a>(
    selectors: &'a [ast::Selector],
    root: &'a Value,
) -> impl Iterator<Item = Selector<'a>> {
    selectors
        .iter()
        .map(|selector| compile_selector(selector, root))
}

fn compile_selector<'a>(selector: &'a ast::Selector, root: &'a Value) -> Selector<'a> {
    match selector {
        ast::Selector::Root => Selector::Root(Root::new(root)),
        ast::Selector::Current => Selector::Current(Current),
        ast::Selector::Key(key) => Selector::Key(ObjectKey::new(key.clone())),
        ast::Selector::Wildcard | ast::Selector::IndexWildcard => Selector::Wildcard(Wildcard),
        ast::Selector::Index(index) => Selector::Index(ArrayIndex::new(*index)),
        ast::Selector::Union(entries) => {
            Selector::Union(compile_selectors(entries, root).collect())
        }
        ast::Selector::Slice(range) => Selector::Slice(Slice::new(SliceRange::new(
            range.start,
            range.end,
            range.step,
        ))),
        ast::Selector::Descendant(selector) => {
            Selector::Descendant(Descendant::new(compile_selector(selector, root)))
        }
        ast::Selector::Filter(expr) => {
            Selector::Filter(Filter::new(compile_filter_expr(expr, root)))
        }
    }
}

fn compile_filter_exprs<'a>(exprs: &'a [ast::FilterExpr], root: &'a Value) -> Vec<FilterExpr<'a>> {
    exprs
        .iter()
        .map(|expr| compile_filter_expr(expr, root))
        .collect()
}

fn compile_filter_expr<'a>(expr: &'a ast::FilterExpr, root: &'a Value) -> FilterExpr<'a> {
    match expr {
        ast::FilterExpr::Not(expr) => FilterExpr::Not(Box::new(compile_filter_expr(expr, root))),
        ast::FilterExpr::Or(exprs) => FilterExpr::Or(compile_filter_exprs(exprs, root)),
        ast::FilterExpr::And(exprs) => FilterExpr::And(compile_filter_exprs(exprs, root)),
        ast::FilterExpr::Exist(path) => FilterExpr::Exist(compile(path, root)),
        ast::FilterExpr::Regex(expr) => FilterExpr::Regex(RegexFilterExpr::new(
            compile_operand(&expr.lhs, root),
            expr.regex.clone(),
        )),
        ast::FilterExpr::Comp(expr) => FilterExpr::Comp(CompFilterExpr::new(
            compile_operand(&expr.lhs, root),
            expr.op.into(),
            compile_operand(&expr.rhs, root),
        )),
    }
}

fn compile_operand<'a>(operand: &'a ast::Operand, root: &'a Value) -> Path<'a> {
    match operand {
        ast::Operand::Value(v) => Selector::Root(Root::new(v)).into(),
        ast::Operand::Path(path) => compile(path, root),
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
