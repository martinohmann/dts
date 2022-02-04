#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::fs::canonicalize;
use std::path::{Path, PathBuf};

pub use encoding::*;
pub use error::*;
pub use sink::Sink;
pub use source::Source;

pub mod de;
mod encoding;
mod error;
pub mod jq;
mod parsers;
pub mod ser;
mod sink;
mod source;
pub mod transform;

trait PathExt {
    fn relative_to<P>(&self, path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>;

    fn relative_to_cwd(&self) -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .and_then(|base| self.relative_to(base))
    }

    fn glob_files(&self, pattern: &str) -> Result<Vec<PathBuf>>;
}

impl<T> PathExt for T
where
    T: AsRef<Path>,
{
    fn relative_to<P>(&self, base: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let (path, base) = (canonicalize(self).ok()?, canonicalize(base).ok()?);
        pathdiff::diff_paths(path, base)
    }

    fn glob_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = self.as_ref().join(pattern);

        glob::glob(&full_pattern.to_string_lossy())
            .map_err(|err| Error::glob_pattern(full_pattern.display(), err))?
            .filter_map(|result| match result {
                Ok(path) => path.is_file().then(|| Ok(path)),
                Err(err) => Some(Err(err.into_error().into())),
            })
            .collect()
    }
}
