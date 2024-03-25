//! GNU Readline-like functionality
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![warn(missing_docs)]
mod errors;
mod readline;

pub use errors::{CrustyLineError, CrustyLineResult};
pub use readline::CrustyLine;
