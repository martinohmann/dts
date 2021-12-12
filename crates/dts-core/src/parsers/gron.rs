use super::{ParseError, ParseErrorKind};
use crate::Result;
use pest::Parser as ParseTrait;
use pest_derive::Parser;
use std::slice::Iter;

#[derive(Parser)]
#[grammar = "parsers/grammars/gron.pest"]
struct GronParser;

/// Parses `Statements` from a `&str`.
pub fn parse(s: &str) -> Result<Statements<'_>, ParseError> {
    let statements = GronParser::parse(Rule::Statements, s)
        .map_err(|e| ParseError::new(ParseErrorKind::Gron, e))?
        .into_iter()
        .filter_map(|pair| match pair.as_rule() {
            Rule::Statement => {
                let mut inner = pair.into_inner();
                // Guaranteed by the grammar that these will exist so unchecked unwrap here is
                // safe.
                let path = inner.next().unwrap().as_str();
                let value = inner.next().unwrap().as_str();

                Some(Statement::new(path, value))
            }
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect();

    Ok(statements)
}

#[derive(Debug, PartialEq)]
pub struct Statement<'a> {
    path: &'a str,
    value: &'a str,
}

impl<'a> Statement<'a> {
    pub fn new(path: &'a str, value: &'a str) -> Self {
        Self { path, value }
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn value(&self) -> &'a str {
        self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct Statements<'a> {
    inner: Vec<Statement<'a>>,
}

impl<'a> Statements<'a> {
    pub fn iter(&self) -> Iter<'a, Statement> {
        self.inner.iter()
    }
}

impl<'a> FromIterator<Statement<'a>> for Statements<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Statement<'a>>,
    {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<'a> IntoIterator for Statements<'a> {
    type Item = Statement<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("foo.bar = \"baz\";").unwrap(),
            Statements::from_iter(vec![Statement::new("foo.bar", "\"baz\"")])
        );
        assert_eq!(
            parse("foo.bar[5].baz = []").unwrap(),
            Statements::from_iter(vec![Statement::new("foo.bar[5].baz", "[]")])
        );
        assert_eq!(
            parse("foo = \"bar\"; baz = 1").unwrap(),
            Statements::from_iter(vec![
                Statement::new("foo", "\"bar\""),
                Statement::new("baz", "1")
            ])
        );
    }
}
