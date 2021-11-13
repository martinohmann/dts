//! Provides serializers and deserializers to transcode between different encodings.

#![deny(missing_docs)]

pub use encoding::*;
pub use error::*;

pub mod args;
pub mod de;
mod encoding;
mod error;
pub mod io;
pub mod ser;
pub mod transform;

/// The type deserializer in this crate deserializes into.
///
/// We use serde_json::Value as our internal deserialization format for now as it should have all
/// the necessary features we need for internal data transformation.
pub type Value = serde_json::Value;

// Serializing a `Value` as `String` can never fail so this function removes the need to wrap the
// string with `Result` which simplifies error handling at the call sites.
pub(crate) fn value_to_string(value: &Value) -> String {
    serde_json::to_string(value).unwrap()
}
