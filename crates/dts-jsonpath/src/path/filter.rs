use super::selector::{JsonPath, PathSelector};
use dts_json::Value;
use regex::Regex;

#[derive(Clone)]
pub enum Filter {
    Not(Box<Filter>),
    Or(Vec<Filter>),
    And(Vec<Filter>),
    Exist(JsonPath),
    Regex(RegexFilter),
    Comp(CompFilter),
}

impl Filter {
    pub(crate) fn matches<'a>(&self, root: &'a Value, current: &'a Value) -> bool {
        match self {
            Filter::Not(filter) => !filter.matches(root, current),
            Filter::Or(filters) => filters.iter().any(|filter| filter.matches(root, current)),
            Filter::And(filters) => filters.iter().all(|filter| filter.matches(root, current)),
            Filter::Exist(path) => !path.select(root, current).is_empty(),
            Filter::Regex(re) => re.matches(root, current),
            Filter::Comp(comp) => comp.matches(root, current),
        }
    }
}

#[derive(Clone)]
pub struct RegexFilter {
    lhs: Operand,
    regex: Regex,
}

impl RegexFilter {
    pub(crate) fn new(lhs: Operand, regex: Regex) -> Self {
        RegexFilter { lhs, regex }
    }

    pub(crate) fn matches<'a>(&self, root: &'a Value, current: &'a Value) -> bool {
        self.lhs.select(root, current).iter().any(|value| {
            value
                .as_str()
                .map(|s| self.regex.is_match(s))
                .unwrap_or_default()
        })
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

#[derive(Clone)]
pub enum Operand {
    Value(Value),
    Path(JsonPath),
}

impl Operand {
    fn select<'a>(&'a self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match self {
            Operand::Value(value) => vec![value],
            Operand::Path(path) => path.select(root, current),
        }
    }
}

#[derive(Clone)]
pub struct CompFilter {
    lhs: Operand,
    op: CompOp,
    rhs: Operand,
}

impl CompFilter {
    pub(crate) fn new(lhs: Operand, op: CompOp, rhs: Operand) -> Self {
        CompFilter { lhs, op, rhs }
    }

    pub(crate) fn matches<'a>(&self, root: &'a Value, current: &'a Value) -> bool {
        let lhs = self.lhs.select(root, current);
        let rhs = self.rhs.select(root, current);

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
