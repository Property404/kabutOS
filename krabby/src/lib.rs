#![no_std]
use core::panic::PanicInfo;
pub mod c_functions;
pub mod drivers;

#[no_mangle]
pub fn snorkel() -> i32 {
    4
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
