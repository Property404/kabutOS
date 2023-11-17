//! KabutOS kernel library
// We're building a kernel, so we don't have access to the standard library
#![no_std]
// Make sure everything's documented by warning when docs are missing
#![warn(missing_docs)]
// Don't allow implicit unsafe operations in `unsafe fn`, so we don't do something unsafe without
// being aware of it. I'm told this will be a hard error in a future version of Rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod c_functions;
pub mod drivers;
pub mod errors;
pub mod functions;
pub mod readline;
pub mod serial;

use core::{fmt::Write, panic::PanicInfo};
use owo_colors::OwoColorize;
use serial::Serial;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // We're already panicking, so let's just ignore these errors
    let _ = writeln!(Serial::new(), "{}", "KERNEL PANIC!".red());
    let _ = writeln!(Serial::new(), "{info}");
    loop {}
}
