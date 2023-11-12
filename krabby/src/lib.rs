#![no_std]
use core::panic::PanicInfo;

#[no_mangle]
pub fn snorkel() -> i32 {
    4
}

extern "C" {
    fn kill_all_humans(dry: bool);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { kill_all_humans(false) };
    loop {}
}
