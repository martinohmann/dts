use super::key::Key;
use crate::{Error, Result};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "transform/flat_key.pest"]
struct FlatKeyParser;

/// Parses a flat object key and returns a vector of key parts.
pub fn parse(key: &str) -> Result<Vec<Key>> {
    let pairs =
        FlatKeyParser::parse(Rule::parts, key).map_err(|e| Error::FlatKey(e.to_string()))?;

    let path = pairs
        .into_iter()
        .filter_map(|pair| match pair.as_rule() {
            Rule::key | Rule::key_escaped => Some(Key::Ident(pair.as_str())),
            Rule::index => Some(Key::Index(pair.as_str().parse::<usize>().unwrap())),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect();

    Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        assert!(parse("foo.[").is_err());
        assert_eq!(parse("foo").unwrap(), vec![Key::Ident("foo")]);
        assert_eq!(
            parse("foo.bar[5].baz").unwrap(),
            vec![
                Key::Ident("foo"),
                Key::Ident("bar"),
                Key::Index(5),
                Key::Ident("baz")
            ]
        );
        assert_eq!(
            parse("foo.bar_baz[0]").unwrap(),
            vec![Key::Ident("foo"), Key::Ident("bar_baz"), Key::Index(0),]
        );
    }
}
