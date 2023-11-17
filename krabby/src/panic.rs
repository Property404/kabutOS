//! Panic handler
use crate::serial::Serial;
use core::{fmt::Write, panic::PanicInfo};
use owo_colors::OwoColorize;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // We're already panicking, so let's just ignore these errors
    let _ = writeln!(Serial::new(), "{}", "KERNEL PANIC!".red());
    let _ = writeln!(Serial::new(), "{info}");
    loop {}
}
