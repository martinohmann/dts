use crate::Encoding;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("error at row index {0}: {1}")]
    AtRowIndex(usize, String),

    #[error("serializing to {0} is not supported")]
    SerializeUnsupported(Encoding),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Json5(#[from] json5::Error),

    #[error(transparent)]
    Hjson(#[from] deser_hjson::Error),

    #[error(transparent)]
    Ron(#[from] ron::Error),

    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    Pickle(#[from] serde_pickle::Error),

    #[error(transparent)]
    QueryString(#[from] serde_qs::Error),

    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),
}

impl Error {
    pub(crate) fn new<T>(message: T) -> Self
    where
        T: AsRef<str>,
    {
        Self::Message(message.as_ref().to_string())
    }
}
