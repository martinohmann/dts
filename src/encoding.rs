//! Supported encodings for serialization and deserialization.

use clap::ValueEnum;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;
use std::path::Path;

/// Encodings supported by this crate.
///
/// Not all of the supported encodings are supported to serialize and deserialize into. Some, like
/// hjson only allow deserialization of encoded data but are not able to serialize back into the
/// original representation.
#[non_exhaustive]
#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Encoding {
    /// JavaScript Object Notation
    Json,
    /// Yet Another Markup Language
    #[clap(alias = "yml")]
    Yaml,
    /// TOML configuration format
    Toml,
    /// ES5 JSON
    Json5,
    /// Comma separated values
    Csv,
    /// URL query string
    #[clap(alias = "qs")]
    QueryString,
    /// Extensible Markup Language
    Xml,
    /// Plaintext document
    #[clap(alias = "txt")]
    Text,
    /// Gron
    Gron,
    /// HCL
    Hcl,
}

// Patterns to detect a source encoding by looking at the first line of input. The patterns are
// lazily constructed upon first usage as they are only needed if there is no other encoding hint
// (e.g. encoding inferred from file extension or explicitly provided on the command line).
//
// These patterns are very basic and will only detect some of the more common first lines. Thus
// they may not match valid pattern for a given encoding on purpose due to ambiguities. For example
// the first line `["foo"]` may be a JSON array or a TOML table header. Make sure to avoid matching
// anything that is ambiguous.
static FIRST_LINES: Lazy<Vec<(Encoding, Regex)>> = Lazy::new(|| {
    vec![
        // XML or HTML start.
        (
            Encoding::Xml,
            Regex::new(
                r#"^(?x:
                    <\?xml\s
                    | \s*<(?:[\w-]+):Envelope\s+
                    | \s*(?i:<!DOCTYPE\s+)
                )"#,
            )
            .unwrap(),
        ),
        // HCL block start of the form
        //
        //   <identifier> [<identifier>|<quoted-string>]* {
        //
        // Expression for matching quoted strings is very basic.
        (
            Encoding::Hcl,
            Regex::new(
                r#"^(?xi:
                    [a-z_][a-z0-9_-]*\s+
                    (?:(?:[a-z_][a-z0-9_-]*|"[^"]*")\s+)*\{
                )"#,
            )
            .unwrap(),
        ),
        // YAML document start or document separator.
        (Encoding::Yaml, Regex::new(r"^(?:%YAML.*|---\s*)$").unwrap()),
        // TOML array of tables or table.
        (
            Encoding::Toml,
            Regex::new(
                r#"^(?xi:
                    # array of tables
                    \[\[\s*[a-z0-9_-]+(?:\s*\.\s*(?:[a-z0-9_-]+|"[^"]*"))*\s*\]\]\s*
                    # table
                    | \[\s*[a-z0-9_-]+(?:\s*\.\s*(?:[a-z0-9_-]+|"[^"]*"))*\s*\]\s*
                )$"#,
            )
            .unwrap(),
        ),
        // JSON object start or array start.
        (
            Encoding::Json,
            Regex::new(r#"^(?:\{\s*(?:"|$)|\[\s*$)"#).unwrap(),
        ),
    ]
});

impl Encoding {
    /// Creates an `Encoding` from a path by looking at the file extension.
    ///
    /// Returns `None` if the extension is absent or if the extension does not match any of the
    /// supported encodings.
    pub fn from_path<P>(path: P) -> Option<Encoding>
    where
        P: AsRef<Path>,
    {
        let ext = path.as_ref().extension()?.to_str()?;

        match ext {
            "json" => Some(Encoding::Json),
            "yaml" | "yml" => Some(Encoding::Yaml),
            "toml" => Some(Encoding::Toml),
            "json5" => Some(Encoding::Json5),
            "csv" => Some(Encoding::Csv),
            "xml" => Some(Encoding::Xml),
            "txt" | "text" => Some(Encoding::Text),
            "hcl" | "tf" => Some(Encoding::Hcl),
            _ => None,
        }
    }

    /// Tries to detect the `Encoding` by looking at the first line of the input.
    ///
    /// Returns `None` if the encoding cannot be detected from the first line.
    pub fn from_first_line(line: &str) -> Option<Encoding> {
        if line.is_empty() {
            // Fast path.
            return None;
        }

        for (encoding, regex) in FIRST_LINES.iter() {
            if regex.is_match(line) {
                return Some(*encoding);
            }
        }

        None
    }

    /// Returns the name of the `Encoding`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Json => "json",
            Encoding::Yaml => "yaml",
            Encoding::Toml => "toml",
            Encoding::Json5 => "json5",
            Encoding::Csv => "csv",
            Encoding::QueryString => "query-string",
            Encoding::Xml => "xml",
            Encoding::Text => "text",
            Encoding::Gron => "gron",
            Encoding::Hcl => "hcl",
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_encoding_from_path() {
        assert_eq!(Encoding::from_path("foo.yaml"), Some(Encoding::Yaml));
        assert_eq!(Encoding::from_path("foo.yml"), Some(Encoding::Yaml));
        assert_eq!(Encoding::from_path("foo.json"), Some(Encoding::Json));
        assert_eq!(Encoding::from_path("foo.json5"), Some(Encoding::Json5));
        assert_eq!(Encoding::from_path("foo.toml"), Some(Encoding::Toml));
        assert_eq!(Encoding::from_path("foo.bak"), None);
        assert_eq!(Encoding::from_path("foo"), None);
    }

    #[test]
    fn test_encoding_from_first_line() {
        // no match
        assert_eq!(Encoding::from_first_line(""), None);
        assert_eq!(Encoding::from_first_line(r#"["foo"]"#), None);

        // match
        assert_eq!(
            Encoding::from_first_line(r#"resource "aws_s3_bucket" "my-bucket" {"#),
            Some(Encoding::Hcl)
        );
        assert_eq!(Encoding::from_first_line("{ "), Some(Encoding::Json));
        assert_eq!(Encoding::from_first_line("[ "), Some(Encoding::Json));
        assert_eq!(
            Encoding::from_first_line(r#"{"foo": 1 }"#),
            Some(Encoding::Json)
        );
        assert_eq!(
            Encoding::from_first_line(r#"[foo .bar."baz".qux]"#),
            Some(Encoding::Toml)
        );
        assert_eq!(
            Encoding::from_first_line(r#"[[foo .bar."baz".qux]] "#),
            Some(Encoding::Toml)
        );
        assert_eq!(Encoding::from_first_line("%YAML 1.2"), Some(Encoding::Yaml));
        assert_eq!(
            Encoding::from_first_line("<!doctype html>"),
            Some(Encoding::Xml)
        );
        assert_eq!(
            Encoding::from_first_line(r#"<?xml version="1.0" ?>"#),
            Some(Encoding::Xml)
        );
        assert_eq!(
            Encoding::from_first_line(
                r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope/" soap:encodingStyle="http://www.w3.org/2003/05/soap-encoding">"#
            ),
            Some(Encoding::Xml)
        );
    }
}
