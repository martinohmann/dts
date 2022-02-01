use super::{
    filter::{CompFilterExpr, FilterExpr, RegexFilterExpr},
    selector::{
        ArrayIndex, Current, Descendant, Filter, ObjectKey, Root, Selector, Slice, Wildcard,
    },
    Path, SliceRange,
};
use crate::parser::ast;
use dts_json::Value;

/// Compiles selectors and a root `Value` into a `Path` instance.
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
            expr.op,
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
