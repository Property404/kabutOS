//! KabutOS kernel library
#![no_std]
#![warn(missing_docs)]
use core::panic::PanicInfo;
pub mod c_functions;
pub mod drivers;
pub mod serial;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
