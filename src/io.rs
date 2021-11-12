//! IO helpers to read from and write to different streams.

use crate::Result;
use std::fs::File;
use std::io::{self, Read, Stdin, Stdout, Write};
use std::path::Path;
use url::Url;

/// A reader that either reads a `File`, `Stdin` or from a boxed reader.
pub enum Reader {
    /// A file reader.
    File(File),
    /// Stdin reader.
    Stdin(Stdin),
    /// Boxed reader.
    Boxed(Box<dyn Read + 'static>),
}

impl Reader {
    /// Creates a new `Reader`.
    ///
    /// If path is `Some`, a `Reader` is constructed that reads from the referenced file.
    ///
    /// ```
    /// use dts::io::Reader;
    /// use tempfile::NamedTempFile;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let file = NamedTempFile::new()?;
    ///
    /// let reader = Reader::new(Some(file.path()));
    /// assert!(matches!(reader, Ok(Reader::File(_))));
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// The path may point to a remote file which will be downloaded.
    ///
    /// Otherwise the returned `Reader` reads from `Stdin`.
    ///
    /// ```
    /// use dts::io::Reader;
    ///
    /// assert!(matches!(Reader::new::<&str>(None), Ok(Reader::Stdin(_))));
    /// ```
    ///
    /// Returns an error if path is `Some` and the file cannot be opened or downloaded.
    pub fn new<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match &path {
            Some(path) => match Url::from_file_path(path) {
                Ok(url) => {
                    if url.scheme() == "file" {
                        Self::from_path(path)
                    } else {
                        Self::from_url(url)
                    }
                }
                Err(_) => Self::from_path(path),
            },
            None => Ok(Self::Stdin(io::stdin())),
        }
    }

    /// Creates a new `Reader` from a local file.
    ///
    /// ```
    /// use dts::io::Reader;
    /// use tempfile::NamedTempFile;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let file = NamedTempFile::new()?;
    ///
    /// let reader = Reader::from_path(file.path());
    /// assert!(matches!(reader, Ok(Reader::File(_))));
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Returns an error if the file cannot be opened.
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self::File(File::open(path)?))
    }

    /// Creates a new `Reader` which reads from a remote url.
    ///
    /// Returns an error if url cannot be parsed or the remote file cannot be downloaded.
    fn from_url<U>(url: U) -> Result<Self>
    where
        U: AsRef<str>,
    {
        let reader = ureq::get(url.as_ref()).call()?.into_reader();

        Ok(Self::Boxed(Box::new(reader)))
    }
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::File(ref mut file) => file.read(buf),
            Self::Stdin(ref mut stdin) => stdin.read(buf),
            Self::Boxed(ref mut reader) => reader.read(buf),
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
    /// use dts::io::Writer;
    /// use tempfile::tempdir;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let dir = tempdir()?;
    /// let writer = Writer::new(Some(dir.path().join("file.txt")));
    /// assert!(matches!(writer, Ok(Writer::File(_))));
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Otherwise the returned `Writer` writes to `Stdout`. A special case is made for a path
    /// equivalent to `Some("-")` which will create a `Stdout` writer as well.
    ///
    /// ```
    /// use dts::io::Writer;
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
