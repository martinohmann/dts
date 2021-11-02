pub mod de;
mod encoding;
mod error;
mod reader;
pub mod ser;
mod writer;

pub use encoding::*;
pub use error::*;
pub use reader::Reader;
pub use writer::Writer;

// We use serde_json::Value as our internal deserialization format for now as it should have all
// the necessary features we need for internal data transformation.
pub type Value = serde_json::Value;
