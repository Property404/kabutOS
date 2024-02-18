//! KabutOS kernel
// We're building a kernel, so we don't have access to the standard library
#![no_std]
#![no_main]
// Make sure everything's documented by warning when docs are missing
//#![warn(missing_docs)]
extern crate alloc;

mod allocator;
pub mod ansi_codes;
mod asm;
pub mod console;
pub mod drivers;
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
    drivers::{ns16550::Ns16550Driver, UartDriver, DRIVERS},
};
use fdt::Fdt;
use owo_colors::OwoColorize;

extern "C" {
    fn enter_supervisor_mode(pmo: isize) -> !;
}

/// Machine pre-mmu entry point
#[no_mangle]
unsafe fn boot(_hart_id: usize, fdt_ptr: *const u8, pmo: isize) {
    // Early init uart
    let uart_driver = Ns16550Driver::new(0x1000_0000 as *mut u8);
    uart_driver.send_str("> early uart ON!\n");

    // Initialize global variables
    uart_driver.send_str("> initializing globals\n");
    unsafe {
        let fdt = Fdt::from_ptr(fdt_ptr).unwrap();
        globals::initialize(fdt);
    }

    // Initialize paging
    uart_driver.send_str("> initializing mmu\n");
    mmu::init_mmu(pmo).unwrap();

    uart_driver.send_str("> fdt\n");
    let fdt_page = fdt_ptr as usize & !(mmu::PAGE_SIZE - 1);
    mmu::identity_map_range(fdt_page, fdt_page + 0x4000).unwrap();

    unsafe {
        uart_driver.send_str("> entering sv mode\n");
        enter_supervisor_mode(pmo);
    }
}

/// Supervisor entry point
#[no_mangle]
unsafe fn kmain() {
    // TODO: Make device mapping dynamic
    mmu::identity_map_range(0x1000_0000, 0x1000_1000).unwrap();
    mmu::identity_map_range(0x2000_0000, 0x2020_0000).unwrap();

    // Initialize drivers
    unsafe { DRIVERS.init(&globals::get().device_tree).unwrap() };

    println!("{}", "Welcome to KabutOS!!!".cyan().bold());

    loop {
        run_console();
    }
}
