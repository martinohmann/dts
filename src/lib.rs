//! Provides serializers and deserializers to transcode between different encodings.

#![warn(missing_docs)]

use std::fs::canonicalize;
use std::path::{Path, PathBuf};

pub use encoding::*;
pub use error::*;
pub use sink::Sink;
pub use source::Source;

pub mod args;
pub mod de;
mod encoding;
mod error;
pub mod ser;
mod sink;
mod source;
pub mod transform;

/// The type deserializer in this crate deserializes into.
///
/// We use serde_json::Value as our internal deserialization format for now as it should have all
/// the necessary features we need for internal data transformation.
pub type Value = serde_json::Value;

/// Serializing a `Value` as `String` can never fail so this function removes the need to wrap the
/// string with `Result` which simplifies error handling at the call sites.
pub(crate) fn value_to_string(value: &Value) -> String {
    serde_json::to_string(value).unwrap()
}

/// Converts value into an array. If the value is of varian `Value::Array`, the wrapped value will
/// be returned. Otherwise the result is a `Vec` which contains one `Value`.
pub(crate) fn value_to_array(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(array) => array.clone(),
        _ => vec![value.clone()],
    }
}

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
        glob::glob(&self.as_ref().join(pattern).to_string_lossy())?
            .filter_map(|result| match result {
                Ok(path) => {
                    if path.is_file() {
                        Some(Ok(path))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err.into_error().into())),
            })
            .collect::<Result<_>>()
    }
}
