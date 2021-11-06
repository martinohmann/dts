use clap::ArgEnum;
use std::fmt;
use std::path::Path;

/// Encodings supported by this crate.
///
/// Not all of the supported encodings are supported to serialize and deserialize into. Some, like
/// hjson only allow deserialization of encoded data but are not able to serialize back into the
/// original representation.
#[non_exhaustive]
#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
pub enum Encoding {
    /// JavaScript Object Notation
    Json,
    /// Yet Another Markup Language
    #[clap(alias = "yml")]
    Yaml,
    /// Rusty Object Notation
    Ron,
    /// TOML configuration format
    Toml,
    /// ES5 JSON
    Json5,
    /// Human readable JSON
    Hjson,
    /// Comma separated values
    Csv,
    /// Python pickle
    Pickle,
    /// URL query string
    #[clap(alias = "qs")]
    QueryString,
    /// Extensible Markup Language
    Xml,
    /// Plaintext document
    #[clap(alias = "txt")]
    Text,
}

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
            "ron" => Some(Encoding::Ron),
            "toml" => Some(Encoding::Toml),
            "json5" => Some(Encoding::Json5),
            "hjson" => Some(Encoding::Hjson),
            "csv" => Some(Encoding::Csv),
            "xml" => Some(Encoding::Xml),
            "txt" | "text" => Some(Encoding::Text),
            _ => None,
        }
    }

    /// Returns the name of the `Encoding`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Json => "json",
            Encoding::Yaml => "yaml",
            Encoding::Ron => "ron",
            Encoding::Toml => "toml",
            Encoding::Json5 => "json5",
            Encoding::Hjson => "hjson",
            Encoding::Csv => "csv",
            Encoding::Pickle => "pickle",
            Encoding::QueryString => "query-string",
            Encoding::Xml => "xml",
            Encoding::Text => "text",
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Chooses a suitable `Encoding` from the provided `Option` values.
///
/// If encoding is `Some` it is returned. Otherwise it attempts to create the `Encoding` from the
/// provided path.
pub fn detect_encoding<P>(encoding: Option<Encoding>, path: Option<P>) -> Option<Encoding>
where
    P: AsRef<Path>,
{
    encoding.or_else(|| path.and_then(Encoding::from_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_encoding_from_path() {
        assert_eq!(Encoding::from_path("foo.yaml"), Some(Encoding::Yaml));
        assert_eq!(Encoding::from_path("foo.yml"), Some(Encoding::Yaml));
        assert_eq!(Encoding::from_path("foo.json"), Some(Encoding::Json));
        assert_eq!(Encoding::from_path("foo.json5"), Some(Encoding::Json5));
        assert_eq!(Encoding::from_path("foo.ron"), Some(Encoding::Ron));
        assert_eq!(Encoding::from_path("foo.toml"), Some(Encoding::Toml));
        assert_eq!(Encoding::from_path("foo.hjson"), Some(Encoding::Hjson));
        assert_eq!(Encoding::from_path("foo.bak"), None);
        assert_eq!(Encoding::from_path("foo"), None);
    }

    #[test]
    fn test_detect_encoding() {
        assert_eq!(detect_encoding::<PathBuf>(None, None), None);
        assert_eq!(
            detect_encoding::<PathBuf>(Some(Encoding::Yaml), None),
            Some(Encoding::Yaml)
        );
        assert_eq!(
            detect_encoding(Some(Encoding::Yaml), Some("foo.json")),
            Some(Encoding::Yaml)
        );
        assert_eq!(
            detect_encoding(None, Some("foo.json")),
            Some(Encoding::Json)
        );
        assert_eq!(detect_encoding(None, Some("foo.bak")), None);
    }
}
