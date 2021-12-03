#![allow(clippy::should_implement_trait)]

pub mod de;
pub mod error;
pub mod number;
mod parser;
pub mod structure;
pub mod value;

pub use de::{from_reader, from_str};
pub use error::*;
pub use value::{Map, Value};
