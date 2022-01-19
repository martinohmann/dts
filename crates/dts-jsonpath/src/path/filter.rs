use super::selector::{JsonPath, Selector, Values};
use regex::Regex;

pub enum Filter {
    Not(Box<Filter>),
    Or(Vec<Filter>),
    And(Vec<Filter>),
    Exist(JsonPath),
    Regex(RegexFilter),
}

impl Filter {
    pub(crate) fn matches<'a>(&self, values: &Values<'a>) -> bool {
        match self {
            Filter::Not(filter) => !filter.matches(values),
            Filter::Or(filters) => filters.iter().any(|filter| filter.matches(values)),
            Filter::And(filters) => filters.iter().all(|filter| filter.matches(values)),
            Filter::Exist(path) => !path.select(values).is_empty(),
            Filter::Regex(re) => re.matches(values),
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

    pub(crate) fn matches<'a>(&self, values: &Values<'a>) -> bool {
        self.path.select(values).iter().any(|value| {
            value
                .as_str()
                .map(|s| self.regex.is_match(s))
                .unwrap_or_default()
        })
    }
}
