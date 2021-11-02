use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, Read, Stdin};
use std::path::Path;

#[non_exhaustive]
pub enum Reader {
    File(File),
    Stdin(Stdin),
}

impl Reader {
    pub fn new<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match &path {
            Some(path) => {
                let file = std::fs::File::open(path)
                    .with_context(|| format!("failed to open file: {}", path.as_ref().display()))?;
                Ok(Self::File(file))
            }
            None => Ok(Self::Stdin(io::stdin())),
        }
    }
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::File(ref mut file) => file.read(buf),
            Self::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}
