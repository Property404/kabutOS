//! KabutOS kernel library
#![no_std]
#![warn(missing_docs)]
use core::panic::PanicInfo;
use owo_colors::OwoColorize;
pub mod c_functions;
pub mod drivers;
pub mod errors;
pub mod readline;
pub mod serial;

use core::fmt::Write;
use serial::Serial;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(Serial::new(), "{}", "KERNEL PANIC!".red());
    let _ = writeln!(Serial::new(), "{info}");
    loop {}
}
