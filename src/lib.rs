mod parse;
mod types;

pub use crate::parse::{deserialize, deserialize_per_line, serialize};
pub use crate::types::{JSONError, Value};
