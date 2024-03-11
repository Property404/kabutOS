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
pub mod frame;
pub mod functions;
pub mod globals;
pub mod handlers;
pub mod mmu;
pub mod panic;
pub mod process;
pub mod readline;
pub mod serial;
pub mod syscalls;
pub mod userspace;
pub mod util;

pub mod prelude {
    pub use super::{print, println};
}

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
unsafe fn boot(hart_id: usize, fdt_ptr: *const u8, pmo: isize) {
    // Only equipped to deal with a single hart, currently
    assert_eq!(hart_id, 0);

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
    mmu::init_page_tables(pmo).unwrap();

    uart_driver.send_str("> fdt\n");
    let fdt_page = fdt_ptr as usize & !(mmu::PAGE_SIZE - 1);
    mmu::map_device(fdt_page, 0x4000).unwrap();

    mmu::init_mmu(pmo).unwrap();

    unsafe {
        uart_driver.send_str("> entering sv mode\n");
        enter_supervisor_mode(pmo);
    }
}

/// Supervisor entry point
#[no_mangle]
unsafe fn kmain() {
    // TODO: set hart ID
    // Set trap frame
    frame::set_kernel_trap_frame(0);

    // Initialize drivers
    unsafe { DRIVERS.init(&globals::get().device_tree).unwrap() };

    unsafe {
        riscv::register::sstatus::set_sum();
    }

    println!("{}", "Welcome to KabutOS!!!".cyan().bold());

    loop {
        run_console();
    }
}
