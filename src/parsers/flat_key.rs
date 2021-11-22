use crate::{Error, Result};
use pest::Parser as ParseTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parsers/grammars/flat_key.pest"]
struct Parser;

#[derive(Debug, PartialEq)]
pub enum KeyPart {
    Index(usize),
    Ident(String),
}

impl ToString for KeyPart {
    fn to_string(&self) -> String {
        match self {
            KeyPart::Index(index) => format!("[{}]", index),
            KeyPart::Ident(key) => {
                let no_escape = key.chars().all(|c| c == '_' || c.is_ascii_alphanumeric());

                if no_escape {
                    key.to_string()
                } else {
                    format!("[\"{}\"]", key.replace("\"", "\\\""))
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct KeyParts {
    inner: Vec<KeyPart>,
}

impl KeyParts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop(&mut self) -> Option<KeyPart> {
        self.inner.pop()
    }

    pub fn push(&mut self, part: KeyPart) {
        self.inner.push(part)
    }

    pub fn push_index(&mut self, index: usize) {
        self.push(KeyPart::Index(index))
    }

    pub fn push_ident<S>(&mut self, ident: S)
    where
        S: ToString,
    {
        self.push(KeyPart::Ident(ident.to_string()))
    }

    pub fn reverse(&mut self) {
        self.inner.reverse()
    }

    pub fn parse(key: &str) -> Result<Self> {
        let parts = Parser::parse(Rule::parts, key)
            .map_err(|e| Error::FlatKey(e.to_string()))?
            .into_iter()
            .filter_map(|pair| match pair.as_rule() {
                Rule::key => Some(KeyPart::Ident(pair.as_str().to_owned())),
                Rule::key_escaped => Some(KeyPart::Ident(pair.as_str().replace("\\\"", "\""))),
                Rule::index => Some(KeyPart::Index(pair.as_str().parse::<usize>().unwrap())),
                Rule::EOI => None,
                _ => unreachable!(),
            })
            .collect();

        Ok(parts)
    }
}

impl ToString for KeyParts {
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

impl FromIterator<KeyPart> for KeyParts {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = KeyPart>,
    {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<'a> IntoIterator for KeyParts {
    type Item = KeyPart;
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
            KeyParts::from_iter(vec![KeyPart::Ident("foo".into())])
        );
        assert_eq!(
            KeyParts::parse("foo.bar[5].baz").unwrap(),
            KeyParts::from_iter(vec![
                KeyPart::Ident("foo".into()),
                KeyPart::Ident("bar".into()),
                KeyPart::Index(5),
                KeyPart::Ident("baz".into())
            ])
        );
        assert_eq!(
            KeyParts::parse("foo.bar_baz[0]").unwrap(),
            KeyParts::from_iter(vec![
                KeyPart::Ident("foo".into()),
                KeyPart::Ident("bar_baz".into()),
                KeyPart::Index(0),
            ])
        );
    }

    #[test]
    fn test_roundtrip() {
        let s = "foo[\"京\\\"\tasdf\"][0]";

        let parsed = KeyParts::parse(s).unwrap();

        let expected = KeyParts::from_iter(vec![
            KeyPart::Ident("foo".into()),
            KeyPart::Ident("京\"\tasdf".into()),
            KeyPart::Index(0),
        ]);

        assert_eq!(parsed, expected);
        assert_eq!(&parsed.to_string(), s);
    }
}
