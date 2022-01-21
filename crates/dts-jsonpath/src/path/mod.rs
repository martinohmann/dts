mod filter;
mod selector;

use crate::parser::ast;
use filter::*;
use selector::*;

pub(crate) use selector::{JsonPath, PathPointer, PathSelector};

pub fn compile(ast: ast::JsonPath) -> JsonPath {
    ast.into_iter().map(compile_selector).collect()
}

fn compile_selector(selector: ast::Selector) -> Box<dyn PathSelector> {
    match selector {
        ast::Selector::Root => Box::new(RootSelector),
        ast::Selector::Current => Box::new(CurrentSelector),
        ast::Selector::Key(key) => Box::new(KeySelector::new(key)),
        ast::Selector::Wildcard => Box::new(WildcardSelector),
        ast::Selector::Index(index) => Box::new(IndexSelector::new(index)),
        ast::Selector::IndexWildcard => Box::new(WildcardSelector),
        ast::Selector::Union(entries) => Box::new(UnionSelector::new(
            entries.into_iter().map(compile_selector),
        )),
        ast::Selector::Slice(s) => Box::new(SliceSelector::new(s.start, s.end, s.step)),
        ast::Selector::Descendant(selector) => {
            Box::new(DescendantSelector::new(compile_selector(*selector)))
        }
        ast::Selector::Filter(expr) => Box::new(FilterSelector::new(compile_filter(expr))),
    }
}

fn compile_filter(expr: ast::FilterExpr) -> Filter {
    match expr {
        ast::FilterExpr::Not(expr) => Filter::Not(Box::new(compile_filter(*expr))),
        ast::FilterExpr::Or(exprs) => Filter::Or(exprs.into_iter().map(compile_filter).collect()),
        ast::FilterExpr::And(exprs) => Filter::And(exprs.into_iter().map(compile_filter).collect()),
        ast::FilterExpr::Exist(path) => Filter::Exist(compile(path)),
        ast::FilterExpr::Regex(expr) => {
            Filter::Regex(RegexFilter::new(expr.lhs.into(), expr.regex))
        }
        ast::FilterExpr::Comp(expr) => {
            Filter::Comp(CompFilter::new(expr.lhs.into(), expr.op, expr.rhs.into()))
        }
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
