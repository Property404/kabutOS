//! Panic handler
use crate::println;
use core::panic::PanicInfo;
use owo_colors::OwoColorize;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", "KERNEL PANIC!".red());
    println!("{info}");
    loop {
        riscv::asm::wfi();
    }
}
