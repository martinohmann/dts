use crate::{Error, Result};
use pest::Parser as ParseTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parsers/grammars/flat_key.pest"]
struct Parser;

#[derive(Debug, PartialEq)]
pub enum KeyPart<'a> {
    Index(usize),
    Ident(&'a str),
}

impl<'a> ToString for KeyPart<'a> {
    fn to_string(&self) -> String {
        match self {
            KeyPart::Index(index) => format!("[{}]", index),
            KeyPart::Ident(key) => {
                let no_escape = key
                    .chars()
                    .all(|c| c == '_' || c.is_numeric() || c.is_alphabetic());

                if no_escape {
                    key.to_string()
                } else {
                    format!("[\"{}\"]", key.escape_default().collect::<String>())
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct KeyParts<'a> {
    inner: Vec<KeyPart<'a>>,
}

impl<'a> KeyParts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop(&mut self) -> Option<KeyPart<'a>> {
        self.inner.pop()
    }

    pub fn push(&mut self, part: KeyPart<'a>) {
        self.inner.push(part)
    }

    pub fn reverse(&mut self) {
        self.inner.reverse()
    }

    pub fn parse(key: &'a str) -> Result<Self> {
        let parts = Parser::parse(Rule::parts, key)
            .map_err(|e| Error::FlatKey(e.to_string()))?
            .into_iter()
            .filter_map(|pair| match pair.as_rule() {
                Rule::key | Rule::key_escaped => Some(KeyPart::Ident(pair.as_str())),
                Rule::index => Some(KeyPart::Index(pair.as_str().parse::<usize>().unwrap())),
                Rule::EOI => None,
                _ => unreachable!(),
            })
            .collect();

        Ok(parts)
    }
}

impl<'a> ToString for KeyParts<'a> {
    fn to_string(&self) -> String {
        self.inner.iter().fold(String::new(), |mut acc, key| {
            let key = key.to_string();
            if !acc.is_empty() && !key.starts_with('[') {
                acc.push('.');
            }
            acc.push_str(&key);
            acc
        })
    }
}

impl<'a> FromIterator<KeyPart<'a>> for KeyParts<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = KeyPart<'a>>,
    {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<'a> IntoIterator for KeyParts<'a> {
    type Item = KeyPart<'a>;
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
        assert!(KeyParts::parse("foo.[").is_err());
        assert_eq!(
            KeyParts::parse("foo").unwrap(),
            KeyParts::from_iter(vec![KeyPart::Ident("foo")])
        );
        assert_eq!(
            KeyParts::parse("foo.bar[5].baz").unwrap(),
            KeyParts::from_iter(vec![
                KeyPart::Ident("foo"),
                KeyPart::Ident("bar"),
                KeyPart::Index(5),
                KeyPart::Ident("baz")
            ])
        );
        assert_eq!(
            KeyParts::parse("foo.bar_baz[0]").unwrap(),
            KeyParts::from_iter(vec![
                KeyPart::Ident("foo"),
                KeyPart::Ident("bar_baz"),
                KeyPart::Index(0),
            ])
        );
    }
}
