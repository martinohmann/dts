//! Provides serializers and deserializers to transcode between different encodings.

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
#[cfg(feature = "custom_value")]
mod number;
mod parsers;
pub mod ser;
mod sink;
mod source;
pub mod transform;
#[cfg(feature = "custom_value")]
mod value;

#[macro_use]
mod macros;

#[cfg(not(feature = "custom_value"))]
pub use serde_json::{to_value, Map, Number, Value};
#[cfg(feature = "custom_value")]
pub use value::{to_value, Map, Number, Value};

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

trait ValueExt {
    /// Converts value into an array. If the value is of variant `Value::Array`, the wrapped value
    /// will be returned. Otherwise the result is a `Vec` which contains the `Value`.
    fn to_array(&self) -> Vec<Value>;

    /// Converts the value to its string representation but ensures that the resulting string is
    /// not quoted.
    fn to_string_unquoted(&self) -> String;

    /// Deep merges `other` into `self`, replacing all values in `other` that were merged into
    /// `self` with `Value::Null`.
    fn deep_merge(&mut self, other: &mut Value);

    /// Returns true if `self` is `Value::Null` or an empty array or map.
    fn is_empty(&self) -> bool;
}
