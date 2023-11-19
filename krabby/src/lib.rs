//! KabutOS kernel library
// We're building a kernel, so we don't have access to the standard library
#![no_std]
// Make sure everything's documented by warning when docs are missing
#![warn(missing_docs)]
// Don't allow implicit unsafe operations in `unsafe fn`, so we don't do something unsafe without
// being aware of it. I'm told this will be a hard error in a future version of Rust
#![deny(unsafe_op_in_unsafe_fn)]

mod panic;

pub mod ansi_codes;
pub mod c_functions;
pub mod drivers;
pub mod errors;
pub mod functions;
pub mod readline;
pub mod serial;
