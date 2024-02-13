//! Panic handler
use crate::println;
use core::panic::PanicInfo;
use owo_colors::OwoColorize;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // We're already panicking, so let's just ignore these errors
    println!("{}", "KERNEL PANIC!".red());
    println!("{info}");
    loop {
        unsafe {
            riscv::asm::wfi();
        }
    }
}
