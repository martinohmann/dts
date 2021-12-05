//! Supported encodings for serialization and deserialization.

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
    /// Gron <https://github.com/TomNomNom/gron>
    Gron,
    /// HCL
    Hcl,
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
            "toml" => Some(Encoding::Toml),
            "json5" => Some(Encoding::Json5),
            "csv" => Some(Encoding::Csv),
            "xml" => Some(Encoding::Xml),
            "txt" | "text" => Some(Encoding::Text),
            "hcl" | "tf" => Some(Encoding::Hcl),
            _ => None,
        }
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
}
