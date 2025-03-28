use crate::{Encoding, Error, PathExt, Result};
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

/// A source for data that needs to be deserialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// Stdin source.
    Stdin,
    /// Local file or directory source.
    Path(PathBuf),
    /// Remote URL source.
    Url(Url),
}

impl Source {
    /// Returns `Some` if the source is a local path, `None` otherwise.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }
    }

    /// Returns `true` if the `Source` is a local path and the path exists on disk and is pointing
    /// at a directory.
    pub fn is_dir(&self) -> bool {
        self.as_path().map(|path| path.is_dir()).unwrap_or(false)
    }

    /// Tries to detect the encoding of the source. Returns `None` if the encoding cannot be
    /// detected.
    pub fn encoding(&self) -> Option<Encoding> {
        match self {
            Self::Stdin => None,
            Self::Path(path) => Encoding::from_path(path),
            Self::Url(url) => Encoding::from_path(url.as_str()),
        }
    }

    /// If source is a local path, this returns sources for all files matching the glob pattern.
    ///
    /// ## Errors
    ///
    /// Returns an error if the sink is not of variant `Sink::Path`, the pattern is invalid or if
    /// there is a `io::Error` while reading the file system.
    pub fn glob_files(&self, pattern: &str) -> Result<Vec<Source>> {
        match self.as_path() {
            Some(path) => Ok(path
                .glob_files(pattern)?
                .iter()
                .map(|path| Self::from(path.as_path()))
                .collect()),
            None => Err(Error::new("not a path source")),
        }
    }

    /// Returns a `SourceReader` to read from the source.
    ///
    /// ## Errors
    ///
    /// May return an error if the source is `Source::Path` and the file cannot be opened of if
    /// source is `Source::Url` and there is an error requesting the remote url.
    pub fn to_reader(&self) -> Result<SourceReader> {
        let reader: Box<dyn io::Read> = match self {
            Self::Stdin => Box::new(io::stdin()),
            Self::Path(path) => Box::new(fs::File::open(path)?),
            Self::Url(url) => Box::new(ureq::get(url.as_ref()).call()?.into_body().into_reader()),
        };

        SourceReader::new(reader, self.encoding())
    }
}

impl From<&str> for Source {
    fn from(s: &str) -> Self {
        if s == "-" {
            Self::Stdin
        } else {
            if let Ok(url) = Url::parse(s) {
                if url.scheme() != "file" {
                    return Self::Url(url);
                }
            }

            Self::Path(PathBuf::from(s))
        }
    }
}

impl From<&Path> for Source {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_path_buf())
    }
}

impl FromStr for Source {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(From::from(s))
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin => write!(f, "<stdin>"),
            Self::Url(url) => url.fmt(f),
            Self::Path(path) => path
                .relative_to_cwd()
                .unwrap_or_else(|| path.clone())
                .display()
                .fmt(f),
        }
    }
}

/// A type that can read from a `Source`. It is able to detect the `Source`'s encoding by looking
/// at the first line of the input.
pub struct SourceReader {
    first_line: Cursor<Vec<u8>>,
    remainder: BufReader<Box<dyn Read>>,
    encoding: Option<Encoding>,
}

impl SourceReader {
    /// Creates a new `SourceReader` for an `io::Read` implementation and an optional encoding
    /// hint.
    ///
    /// Reads the first line from `reader` upon creation.
    ///
    /// ## Errors
    ///
    /// Returns an error if reading the first line from the reader fails.
    pub fn new(reader: Box<dyn Read>, encoding: Option<Encoding>) -> Result<SourceReader> {
        let mut remainder = BufReader::new(reader);
        let mut buf = Vec::new();

        remainder.read_until(b'\n', &mut buf)?;

        let first_line = Cursor::new(buf);

        Ok(SourceReader {
            first_line,
            remainder,
            encoding,
        })
    }

    /// Tries to detect the encoding of the source. If the source provides an encoding hint it is
    /// returned as is. Otherwise the `SourceReader` attempts to detect the encoding based on the
    /// contents of the first line of the input data.
    ///
    /// Returns `None` if the encoding cannot be detected.
    pub fn encoding(&self) -> Option<Encoding> {
        self.encoding.or_else(|| {
            std::str::from_utf8(self.first_line.get_ref())
                .ok()
                .and_then(Encoding::from_first_line)
        })
    }
}

impl Read for SourceReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.first_line.position() < self.first_line.get_ref().len() as u64 {
            self.first_line.read(buf)
        } else {
            self.remainder.read(buf)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_str() {
        assert_eq!(Source::from_str("-"), Ok(Source::Stdin));
        assert_eq!(
            Source::from_str("foo.json"),
            Ok(Source::Path(PathBuf::from("foo.json")))
        );
        assert_eq!(
            Source::from_str("http://localhost/foo.json"),
            Ok(Source::Url(
                Url::from_str("http://localhost/foo.json").unwrap()
            ))
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(Source::from("-").encoding(), None);
        assert_eq!(Source::from("foo").encoding(), None);
        assert_eq!(Source::from("foo.json").encoding(), Some(Encoding::Json));
        assert_eq!(
            Source::from("http://localhost/bar.yaml").encoding(),
            Some(Encoding::Yaml)
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(&Source::Stdin.to_string(), "<stdin>");
        assert_eq!(&Source::from("Cargo.toml").to_string(), "Cargo.toml");
        assert_eq!(
            &Source::from(std::fs::canonicalize("src/lib.rs").unwrap().as_path()).to_string(),
            "src/lib.rs"
        );
        assert_eq!(
            &Source::from("/non-existent/path").to_string(),
            "/non-existent/path"
        );
        assert_eq!(
            &Source::from("http://localhost/bar.yaml").to_string(),
            "http://localhost/bar.yaml",
        );
    }

    #[test]
    fn test_glob_files() {
        assert!(
            Source::from("src/")
                .glob_files("*.rs")
                .unwrap()
                .contains(&Source::from("src/lib.rs"))
        );
        assert!(Source::from("-").glob_files("*.json").is_err());
        assert!(
            Source::from("http://localhost/")
                .glob_files("*.json")
                .is_err(),
        );
        assert!(matches!(
            Source::from("src/").glob_files("***"),
            Err(Error::GlobPatternError { .. })
        ));
    }

    #[test]
    fn test_source_reader() {
        let input = Cursor::new("---\nfoo: bar\n");
        let mut reader = SourceReader::new(Box::new(input), None).unwrap();

        assert_eq!(reader.encoding(), Some(Encoding::Yaml));

        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();

        assert_eq!(&buf, "---\nfoo: bar\n");
    }
}
