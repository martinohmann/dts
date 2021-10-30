use clap::ArgEnum;
use std::path::{Path, PathBuf};

pub mod de;
pub mod ser;

#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
pub enum Encoding {
    Json,
    #[clap(alias = "yml")]
    Yaml,
    Ron,
    Toml,
    Json5,
    Hjson,
}

impl Encoding {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Encoding> {
        let ext = path.as_ref().extension()?.to_str()?;

        Self::from_extension(ext)
    }

    fn from_extension(ext: &str) -> Option<Encoding> {
        match ext {
            "json" => Some(Encoding::Json),
            "yaml" | "yml" => Some(Encoding::Yaml),
            "ron" => Some(Encoding::Ron),
            "toml" => Some(Encoding::Toml),
            "json5" => Some(Encoding::Json5),
            "hjson" => Some(Encoding::Hjson),
            _ => None,
        }
    }
}

pub fn detect_encoding(encoding: Option<Encoding>, path: Option<&PathBuf>) -> Option<Encoding> {
    match encoding {
        Some(encoding) => Some(encoding),
        None => match &path {
            Some(path) => Encoding::from_path(path),
            None => None,
        },
    }
}
