use crate::Encoding;
use anyhow::{bail, Result};
use erased_serde::Serialize;

#[derive(Debug)]
pub struct SerializeOptions {
    pub encoding: Encoding,
    pub pretty: bool,
    pub newline: bool,
}

pub struct Serializer {
    opts: SerializeOptions,
}

impl Serializer {
    pub fn new(opts: SerializeOptions) -> Self {
        Self { opts }
    }

    pub fn serialize<W>(&self, writer: W, value: Box<dyn Serialize>) -> Result<()>
    where
        W: std::io::Write,
    {
        let mut writer = writer;

        match &self.opts.encoding {
            Encoding::Yaml => serde_yaml::to_writer(writer.by_ref(), &value)?,
            Encoding::Json | Encoding::Json5 => {
                if self.opts.pretty {
                    serde_json::to_writer_pretty(writer.by_ref(), &value)?
                } else {
                    serde_json::to_writer(writer.by_ref(), &value)?
                }
            }
            Encoding::Ron => {
                if self.opts.pretty {
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
                let s = if self.opts.pretty {
                    toml::ser::to_string_pretty(&value)?
                } else {
                    toml::ser::to_string(&value)?
                };
                writer.by_ref().write_all(s.as_bytes())?
            }
            encoding => bail!("serializing to {:?} is not supported", encoding),
        };

        if self.opts.newline {
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}
