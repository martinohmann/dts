#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;

pub use error::{Error, Result};
pub use parser::parse;
