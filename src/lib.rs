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

    /// If value is of variant `Value::Object` or `Value::Array`, convert it to a `Value::String`
    /// containing the json encoded string representation of the value.
    fn stringify_collections(&self) -> Value;
}

impl ValueExt for Value {
    fn to_array(&self) -> Vec<Value> {
        match self {
            Value::Array(array) => array.clone(),
            _ => vec![self.clone()],
        }
    }

    fn stringify_collections(&self) -> Value {
        if self.is_array() || self.is_object() {
            Value::String(self.to_string())
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn stringify_collections() {
        assert_eq!(
            json!({"foo": "bar"}).stringify_collections(),
            json!(r#"{"foo":"bar"}"#)
        );
        assert_eq!(
            json!(["foo", "bar"]).stringify_collections(),
            json!(r#"["foo","bar"]"#)
        );
        assert_eq!(json!("foo").stringify_collections(), json!("foo"));
        assert_eq!(json!(true).stringify_collections(), json!(true));
        assert_eq!(json!(1).stringify_collections(), json!(1));
        assert_eq!(Value::Null.stringify_collections(), Value::Null);
    }
}
