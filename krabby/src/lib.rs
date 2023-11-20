//! KabutOS kernel library
// We're building a kernel, so we don't have access to the standard library
#![no_std]
// Make sure everything's documented by warning when docs are missing
#![warn(missing_docs)]

mod errors;
mod panic;

pub mod ansi_codes;
pub mod c_functions;
pub mod drivers;
pub mod functions;
pub mod readline;
pub mod serial;

pub use errors::{KernelError, KernelResult};
