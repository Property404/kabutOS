#![no_std]
use core::panic::PanicInfo;
pub mod c_functions;
pub mod drivers;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
