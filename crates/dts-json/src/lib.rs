#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod number;
mod value;

#[macro_use]
mod macros;

pub use error::*;
pub use value::{to_value, Map, Number, Value};
