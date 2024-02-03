//! KabutOS kernel
// We're building a kernel, so we don't have access to the standard library
#![no_std]
#![no_main]
// Make sure everything's documented by warning when docs are missing
#![warn(missing_docs)]

pub mod ansi_codes;
pub mod console;
pub mod drivers;
mod entry;
pub mod errors;
pub mod functions;
pub mod globals;
pub mod handlers;
pub mod panic;
pub mod readline;
pub mod serial;

pub use crate::errors::{KernelError, KernelResult};
use crate::{
    ansi_codes::CLEAR_SCREEN,
    console::run_console,
    drivers::{ns16550::Ns16550Driver, DRIVERS},
    serial::Serial,
};
use core::fmt::Write;
use fdt::Fdt;
use owo_colors::OwoColorize;

/// Kernel entry point
#[no_mangle]
unsafe fn kmain(_hart_id: usize, fdt_ptr: *const u8) {
    // Initialize drivers
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    unsafe { DRIVERS.uart = Some(uart_driver) };

    // Initialize global variables
    unsafe {
        let fdt = Fdt::from_ptr(fdt_ptr).unwrap();
        globals::initialize(fdt);
    }

    let mut serial = Serial::new();
    writeln!(
        serial,
        "{CLEAR_SCREEN}{}",
        "Welcome to KabutOS!!!".cyan().bold()
    )
    .unwrap();
    writeln!(serial, "Device tree is @ {fdt_ptr:p}").unwrap();

    loop {
        run_console();
    }
}
