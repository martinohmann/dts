#![deny(missing_docs)]
use crate::Result;
use std::fs::File;
use std::io::{self, Read, Stdin};
use std::path::Path;

/// A reader that either reads a `File` or `Stdin`.
pub enum Reader {
    /// A file reader.
    File(File),
    /// Stdin reader.
    Stdin(Stdin),
}

impl Reader {
    /// Creates a new `Reader`.
    ///
    /// If path is `Some`, a `Reader` is constructed that reads from the referenced file. Otherwise
    /// the returned `Reader` reads from `Stdin`.
    ///
    /// Returns an error if path is `Some` and the file cannot be opened.
    pub fn new<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match &path {
            Some(path) => Ok(Self::File(File::open(path)?)),
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
