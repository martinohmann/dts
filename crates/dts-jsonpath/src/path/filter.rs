use super::{CompOp, JsonPath, Select};
use dts_json::Value;
use regex::Regex;

#[derive(Clone)]
pub enum FilterExpr<'a> {
    Not(Box<FilterExpr<'a>>),
    Or(Vec<FilterExpr<'a>>),
    And(Vec<FilterExpr<'a>>),
    Exist(JsonPath<'a>),
    Regex(RegexFilterExpr<'a>),
    Comp(CompFilterExpr<'a>),
}

impl<'a> FilterExpr<'a> {
    pub(crate) fn matches(&self, value: &'a Value) -> bool {
        match self {
            FilterExpr::Not(filter) => !filter.matches(value),
            FilterExpr::Or(filters) => filters.iter().any(|filter| filter.matches(value)),
            FilterExpr::And(filters) => filters.iter().all(|filter| filter.matches(value)),
            FilterExpr::Exist(path) => !path.select(value).is_empty(),
            FilterExpr::Regex(re) => re.matches(value),
            FilterExpr::Comp(comp) => comp.matches(value),
        }
    }
}

#[derive(Clone)]
pub struct RegexFilterExpr<'a> {
    lhs: JsonPath<'a>,
    regex: Regex,
}

impl<'a> RegexFilterExpr<'a> {
    pub(crate) fn new(lhs: JsonPath<'a>, regex: Regex) -> Self {
        RegexFilterExpr { lhs, regex }
    }

    pub(crate) fn matches(&self, value: &'a Value) -> bool {
        self.lhs.select(value).iter().any(|value| {
            value
                .as_str()
                .map(|s| self.regex.is_match(s))
                .unwrap_or_default()
        })
    }
}

#[derive(Clone)]
pub struct CompFilterExpr<'a> {
    lhs: JsonPath<'a>,
    op: CompOp,
    rhs: JsonPath<'a>,
}

impl<'a> CompFilterExpr<'a> {
    pub(crate) fn new(lhs: JsonPath<'a>, op: CompOp, rhs: JsonPath<'a>) -> Self {
        CompFilterExpr { lhs, op, rhs }
    }

    pub(crate) fn matches(&self, value: &'a Value) -> bool {
        let lhs = self.lhs.select(value);
        let rhs = self.rhs.select(value);

        match &self.op {
            CompOp::Eq => eq(&lhs, &rhs),
            CompOp::NotEq => !eq(&lhs, &rhs),
            CompOp::Less => less(&lhs, &rhs),
            CompOp::LessEq => less(&lhs, &rhs) || eq(&lhs, &rhs),
            CompOp::Greater => less(&rhs, &lhs),
            CompOp::GreaterEq => less(&rhs, &lhs) || eq(&lhs, &rhs),
            CompOp::In => contains(&lhs, &rhs),
        }
    }
}

fn eq(lhs: &[&Value], rhs: &[&Value]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    lhs.iter().zip(rhs).all(|(a, b)| a.eq(b))
}

fn contains(lhs: &[&Value], rhs: &[&Value]) -> bool {
    if lhs.is_empty() {
        return false;
    }

    match rhs.get(0) {
        Some(Value::Array(array)) => lhs.iter().any(|l| array.contains(l)),
        Some(Value::Object(object)) => lhs.iter().any(|l| object.values().any(|r| l.eq(&r))),
        _ => false,
    }
}

fn less(lhs: &[&Value], rhs: &[&Value]) -> bool {
    if lhs.len() != 1 && rhs.len() != 1 {
        return false;
    }

    match (lhs.get(0), rhs.get(0)) {
        (Some(Value::Number(lhs)), Some(Value::Number(rhs))) => lhs
            .as_f64()
            .and_then(|l| rhs.as_f64().map(|r| l < r))
            .unwrap_or_default(),
        _ => false,
    }
}
