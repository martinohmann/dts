use clap::ArgEnum;
use std::fmt;
use std::path::Path;

#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
pub enum Encoding {
    Json,
    #[clap(alias = "yml")]
    Yaml,
    Ron,
    Toml,
    Json5,
    Hjson,
    Csv,
    Tsv,
    Pickle,
    #[clap(alias = "qs")]
    QueryString,
}

impl Encoding {
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
            "tsv" => Some(Encoding::Tsv),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Json => "json",
            Encoding::Yaml => "yaml",
            Encoding::Ron => "ron",
            Encoding::Toml => "toml",
            Encoding::Json5 => "json5",
            Encoding::Hjson => "hjson",
            Encoding::Csv => "csv",
            Encoding::Tsv => "tsv",
            Encoding::Pickle => "pickle",
            Encoding::QueryString => "query-string",
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

pub fn detect_encoding<P>(encoding: Option<Encoding>, path: Option<P>) -> Option<Encoding>
where
    P: AsRef<Path>,
{
    match encoding {
        Some(encoding) => Some(encoding),
        None => match &path {
            Some(path) => Encoding::from_path(path),
            None => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
