#![no_std]
use core::panic::PanicInfo;
pub mod driver;
pub mod helpers;
pub mod uart;

#[no_mangle]
pub fn snorkel() -> i32 {
    4
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
