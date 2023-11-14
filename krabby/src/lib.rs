//! KabutOS kernel library
#![no_std]
#![warn(missing_docs)]
use core::panic::PanicInfo;
pub mod c_functions;
pub mod drivers;
pub mod serial;

use core::fmt::Write;
use serial::Serial;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(Serial::new(), "\x1b[31mKERNEL PANIC!\x1b[0m {}", info);
    loop {}
}
