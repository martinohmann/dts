use crate::{Encoding, Value};
use anyhow::{bail, Result};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SerializeOptions {
    pub pretty: bool,
    pub newline: bool,
}

pub struct Serializer {
    encoding: Encoding,
}

impl Serializer {
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }

    pub fn serialize<W>(&self, writer: W, value: Value, opts: SerializeOptions) -> Result<()>
    where
        W: std::io::Write,
    {
        let mut writer = writer;

        match &self.encoding {
            Encoding::Yaml => serde_yaml::to_writer(writer.by_ref(), &value)?,
            Encoding::Json | Encoding::Json5 => {
                if opts.pretty {
                    serde_json::to_writer_pretty(writer.by_ref(), &value)?
                } else {
                    serde_json::to_writer(writer.by_ref(), &value)?
                }
            }
            Encoding::Ron => {
                if opts.pretty {
                    ron::ser::to_writer_pretty(
                        writer.by_ref(),
                        &value,
                        ron::ser::PrettyConfig::default(),
                    )?
                } else {
                    ron::ser::to_writer(writer.by_ref(), &value)?
                }
            }
            Encoding::Toml => {
                let s = if opts.pretty {
                    toml::ser::to_string_pretty(&value)?
                } else {
                    toml::ser::to_string(&value)?
                };
                writer.by_ref().write_all(s.as_bytes())?
            }
            Encoding::Csv => {
                if !value.is_array() {
                    bail!(
                        "serializing to {:?} requires the input data to be an array",
                        &self.encoding
                    )
                }

                let mut csv_writer = csv::Writer::from_writer(writer.by_ref());

                for row in value.as_array().unwrap() {
                    csv_writer.serialize(row)?;
                }
            }
            Encoding::Tsv => {
                if !value.is_array() {
                    bail!(
                        "serializing to {:?} requires the input data to be an array",
                        &self.encoding
                    )
                }

                let mut tsv_writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(writer.by_ref());

                for row in value.as_array().unwrap() {
                    tsv_writer.serialize(row)?;
                }
            }
            encoding => bail!("serializing to {:?} is not supported", encoding),
        };

        if opts.newline {
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}
