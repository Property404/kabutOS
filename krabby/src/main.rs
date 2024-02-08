//! KabutOS kernel
// We're building a kernel, so we don't have access to the standard library
#![no_std]
#![no_main]
// Make sure everything's documented by warning when docs are missing
//#![warn(missing_docs)]

pub mod ansi_codes;
pub mod console;
pub mod drivers;
mod entry;
pub mod errors;
pub mod functions;
pub mod globals;
pub mod handlers;
pub mod mmu;
pub mod panic;
pub mod readline;
pub mod serial;

pub use crate::errors::{KernelError, KernelResult};
use crate::{
    console::run_console,
    drivers::{ns16550::Ns16550Driver, DRIVERS},
    serial::Serial,
};
use core::fmt::Write;
use fdt::Fdt;
use owo_colors::OwoColorize;

extern "C" {
    fn enter_supervisor_mode() -> !;
}

/// Kernel entry point
#[no_mangle]
unsafe fn kmain(_hart_id: usize, fdt_ptr: *const u8) {
    // Early init uart
    let uart_driver = Ns16550Driver::new(0x1000_0000 as *mut u8);
    unsafe { DRIVERS.uart = Some(uart_driver) };
    let _ = writeln!(Serial::new(), "Early UART initialization on!",);
    let _ = writeln!(Serial::new(), "dt: {fdt_ptr:?}");

    // Initialize global variables
    unsafe {
        let fdt = Fdt::from_ptr(fdt_ptr).unwrap();
        globals::initialize(fdt);
    }

    // Initialize paging
    mmu::init_mmu().unwrap();

    mmu::identity_map_range(fdt_ptr as usize, fdt_ptr as usize + 0x4000).unwrap();

    unsafe {
        enter_supervisor_mode();
    }
}

/// Supervisor entry point
#[no_mangle]
unsafe fn svmain() {
    // Initialize drivers
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    unsafe { DRIVERS.uart = Some(uart_driver) };

    let mut serial = Serial::new();
    writeln!(serial, "{}", "Welcome to KabutOS!!!".cyan().bold()).unwrap();

    loop {
        run_console();
    }
}
