//! Provides serializers and deserializers to transcode between different encodings.

#![deny(missing_docs)]

pub use encoding::*;
pub use error::*;

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
