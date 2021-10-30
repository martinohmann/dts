use clap::ArgEnum;
use std::path::Path;

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
    Csv,
    Tsv,
}

impl Encoding {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Encoding> {
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
