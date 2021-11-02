use crate::Result;
use std::fs::File;
use std::io::{self, Stdout, Write};
use std::path::Path;

#[non_exhaustive]
pub enum Writer {
    File(File),
    Stdout(Stdout),
}

impl Writer {
    pub fn new<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match &path {
            Some(path) => match path.as_ref().to_str() {
                Some("-") => Ok(Self::Stdout(io::stdout())),
                _ => {
                    let file = std::fs::File::create(path)?;
                    Ok(Self::File(file))
                }
            },
            None => Ok(Self::Stdout(io::stdout())),
        }
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::File(ref mut file) => file.write(buf),
            Self::Stdout(ref mut stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::File(ref mut file) => file.flush(),
            Self::Stdout(ref mut stdout) => stdout.flush(),
        }
    }
}
