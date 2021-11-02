#![deny(missing_docs)]
use crate::Result;
use std::fs::File;
use std::io::{self, Stdout, Write};
use std::path::Path;

/// A reader that either writes to a `File` or `Stdout`.
pub enum Writer {
    /// A file writer.
    File(File),
    /// Stdout writer.
    Stdout(Stdout),
}

impl Writer {
    /// Creates a new `Writer`.
    ///
    /// If path is `Some`, a `Writer` is constructed that writes to the referenced file. Otherwise
    /// the returned `Writer` writes to `Stdout`. A special case is made for a path equivalent to
    /// `Some("-")` which will create a `Stdout` writer.
    ///
    /// Returns an error if path is `Some` and the file cannot be created.
    pub fn new<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match &path {
            Some(path) => match path.as_ref().to_str() {
                Some("-") => Ok(Self::Stdout(io::stdout())),
                _ => Ok(Self::File(File::create(path)?)),
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
