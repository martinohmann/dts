mod filter;
mod selector;

use crate::parser::ast;
use dts_json::Value;
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
            let lhs = match expr.matchable {
                ast::RegexMatchable::String(s) => Comparable::Value(s.into()),
                ast::RegexMatchable::Path(path) => Comparable::Path(compile(path)),
            };

            Filter::Regex(RegexFilter::new(lhs, expr.regex))
        }
        ast::FilterExpr::Contain(expr) => {
            let lhs = match expr.containable {
                ast::Containable::String(s) => Comparable::Value(s.into()),
                ast::Containable::Number(f) => Comparable::Value(f.into()),
                ast::Containable::Path(path) => Comparable::Path(compile(path)),
            };

            let rhs = match expr.container {
                ast::Container::Array(array) => Comparable::Value(array.into()),
                ast::Container::Object(object) => Comparable::Value(object.into()),
                ast::Container::Path(path) => Comparable::Path(compile(path)),
            };

            Filter::Comp(CompFilter::new(lhs, CompOp::In, rhs))
        }
        ast::FilterExpr::Comp(expr) => {
            let lhs = match expr.lhs {
                ast::Comparable::String(s) => Comparable::Value(s.into()),
                ast::Comparable::Number(f) => Comparable::Value(f.into()),
                ast::Comparable::Boolean(b) => Comparable::Value(b.into()),
                ast::Comparable::Null => Comparable::Value(Value::Null),
                ast::Comparable::Path(path) => Comparable::Path(compile(path)),
            };

            let rhs = match expr.rhs {
                ast::Comparable::String(s) => Comparable::Value(s.into()),
                ast::Comparable::Number(f) => Comparable::Value(f.into()),
                ast::Comparable::Boolean(b) => Comparable::Value(b.into()),
                ast::Comparable::Null => Comparable::Value(Value::Null),
                ast::Comparable::Path(path) => Comparable::Path(compile(path)),
            };

            let op = match expr.op {
                ast::CompOp::Eq => CompOp::Eq,
                ast::CompOp::NotEq => CompOp::NotEq,
                ast::CompOp::LessEq => CompOp::LessEq,
                ast::CompOp::Less => CompOp::Less,
                ast::CompOp::GreaterEq => CompOp::GreaterEq,
                ast::CompOp::Greater => CompOp::Greater,
            };

            Filter::Comp(CompFilter::new(lhs, op, rhs))
        }
    }
}
