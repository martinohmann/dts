use super::selector::{JsonPath, PathPointer, PathSelector};
use regex::Regex;

pub enum Filter {
    Not(Box<Filter>),
    Or(Vec<Filter>),
    And(Vec<Filter>),
    Exist(JsonPath),
    Regex(RegexFilter),
}

impl Filter {
    pub(crate) fn matches<'a>(&self, pointer: &PathPointer<'a>) -> bool {
        match self {
            Filter::Not(filter) => !filter.matches(pointer),
            Filter::Or(filters) => filters.iter().any(|filter| filter.matches(pointer)),
            Filter::And(filters) => filters.iter().all(|filter| filter.matches(pointer)),
            Filter::Exist(path) => !path.select(pointer).is_empty(),
            Filter::Regex(re) => re.matches(pointer),
        }
    }
}

pub struct RegexFilter {
    path: JsonPath,
    regex: Regex,
}

impl RegexFilter {
    pub(crate) fn new(path: JsonPath, regex: Regex) -> Self {
        RegexFilter { path, regex }
    }

    pub(crate) fn matches<'a>(&self, pointer: &PathPointer<'a>) -> bool {
        self.path.select(pointer).iter().any(|value| {
            value
                .as_str()
                .map(|s| self.regex.is_match(s))
                .unwrap_or_default()
        })
    }
}
