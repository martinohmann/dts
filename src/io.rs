//! IO helpers to read from and write to different streams.

use crate::Result;
use std::fs::File;
use std::io::{self, Read, Stdin, Stdout, Write};
use std::path::Path;

/// A reader that either reads a `File` or `Stdin`.
#[derive(Debug)]
pub enum Reader {
    /// A file reader.
    File(File),
    /// Stdin reader.
    Stdin(Stdin),
}

impl Reader {
    /// Creates a new `Reader`.
    ///
    /// If path is `Some`, a `Reader` is constructed that reads from the referenced file.
    ///
    /// ```
    /// # use dts::io::Reader;
    /// use tempfile::NamedTempFile;
    ///
    /// let file = NamedTempFile::new().unwrap();
    ///
    /// let reader = Reader::new(Some(file.path()));
    /// assert!(matches!(reader, Ok(Reader::File(_))));
    /// ```
    ///
    /// Otherwise the returned `Reader` reads from `Stdin`.
    ///
    /// ```
    /// # use dts::io::Reader;
    ///
    /// assert!(matches!(Reader::new::<&str>(None), Ok(Reader::Stdin(_))));
    /// ```
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

/// A writer that either writes to a `File` or `Stdout`.
#[derive(Debug)]
pub enum Writer {
    /// A file writer.
    File(File),
    /// Stdout writer.
    Stdout(Stdout),
}

impl Writer {
    /// Creates a new `Writer`.
    ///
    /// If path is `Some`, a `Writer` is constructed that writes to the referenced file.
    ///
    /// ```
    /// # use dts::io::Writer;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let writer = Writer::new(Some(dir.path().join("file.txt")));
    /// assert!(matches!(writer, Ok(Writer::File(_))));
    /// ```
    ///
    /// Otherwise the returned `Writer` writes to `Stdout`. A special case is made for a path
    /// equivalent to `Some("-")` which will create a `Stdout` writer as well.
    ///
    /// ```
    /// # use dts::io::Writer;
    ///
    /// assert!(matches!(Writer::new::<&str>(None), Ok(Writer::Stdout(_))));
    /// assert!(matches!(Writer::new(Some("-")), Ok(Writer::Stdout(_))));
    /// ```
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
