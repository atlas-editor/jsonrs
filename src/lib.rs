mod parse;
mod types;

pub use crate::parse::{deserialize, serialize};
pub use crate::types::{JSONError, Value};
