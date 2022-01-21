use super::selector::{JsonPath, PathPointer, PathSelector};
use dts_json::Value;
use regex::Regex;

pub enum Filter {
    Not(Box<Filter>),
    Or(Vec<Filter>),
    And(Vec<Filter>),
    Exist(JsonPath),
    Regex(RegexFilter),
    Comp(CompFilter),
}

impl Filter {
    pub(crate) fn matches<'a>(&self, pointer: &PathPointer<'a>) -> bool {
        match self {
            Filter::Not(filter) => !filter.matches(pointer),
            Filter::Or(filters) => filters.iter().any(|filter| filter.matches(pointer)),
            Filter::And(filters) => filters.iter().all(|filter| filter.matches(pointer)),
            Filter::Exist(path) => !path.select(pointer).is_empty(),
            Filter::Regex(re) => re.matches(pointer),
            Filter::Comp(comp) => comp.matches(pointer),
        }
    }
}

pub struct RegexFilter {
    lhs: Comparable,
    regex: Regex,
}

impl RegexFilter {
    pub(crate) fn new(lhs: Comparable, regex: Regex) -> Self {
        RegexFilter { lhs, regex }
    }

    pub(crate) fn matches<'a>(&self, pointer: &PathPointer<'a>) -> bool {
        self.lhs.select(pointer).iter().any(|value| {
            value
                .as_str()
                .map(|s| self.regex.is_match(s))
                .unwrap_or_default()
        })
    }
}

pub enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    In,
}

pub enum Comparable {
    Value(Value),
    Path(JsonPath),
}

impl Comparable {
    fn select<'a>(&'a self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match self {
            Comparable::Value(value) => vec![value],
            Comparable::Path(path) => path.select(pointer),
        }
    }
}

pub struct CompFilter {
    lhs: Comparable,
    op: CompOp,
    rhs: Comparable,
}

impl CompFilter {
    pub(crate) fn new(lhs: Comparable, op: CompOp, rhs: Comparable) -> Self {
        CompFilter { lhs, op, rhs }
    }

    pub(crate) fn matches<'a>(&self, pointer: &PathPointer<'a>) -> bool {
        let lhs = self.lhs.select(pointer);
        let rhs = self.rhs.select(pointer);

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

fn eq(lhs: &Vec<&Value>, rhs: &Vec<&Value>) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    lhs.iter().zip(rhs).all(|(a, b)| a.eq(b))
}

fn contains(lhs: &Vec<&Value>, rhs: &Vec<&Value>) -> bool {
    if lhs.is_empty() {
        return false;
    }

    match rhs.get(0) {
        Some(Value::Array(array)) => lhs.iter().any(|l| array.contains(l)),
        Some(Value::Object(object)) => lhs.iter().any(|l| object.values().any(|r| l.eq(&r))),
        _ => false,
    }
}

fn less(lhs: &Vec<&Value>, rhs: &Vec<&Value>) -> bool {
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
