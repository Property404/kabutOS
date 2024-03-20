//! KabutOS kernel
// We're building a kernel, so we don't have access to the standard library
#![no_std]
#![no_main]

use core::time::Duration;
use fdt::Fdt;
use krabby::{
    console::run_console,
    drivers::{ns16550::Ns16550Driver, UartDriver, DRIVERS},
    frame, globals, mmu,
    mmu::PAGE_SIZE,
    prelude::*,
    timer,
    util::*,
};
use owo_colors::OwoColorize;

extern "C" {
    fn enter_supervisor_mode(pmo: isize) -> !;
}

/// Machine pre-mmu entry point
#[no_mangle]
unsafe fn boot(hart_id: HartId, fdt_ptr: *const u8, pmo: isize) {
    // Boot should only see the hart 0
    assert!(hart_id.is_zero());

    // Early init uart
    let mut uart_driver = Ns16550Driver::new(0x1000_0000 as *mut u8);
    uart_driver.send_str("> early uart ON!\n");

    // Initialize paging
    uart_driver.send_str("> initializing page_tabels\n");
    mmu::init_page_tables(pmo).unwrap();

    // Initialize global variables
    uart_driver.send_str("> fdt\n");
    let fdt_size = unsafe { Fdt::from_ptr(fdt_ptr).unwrap().total_size() };
    let fdt_ptr = mmu::map_device(
        align_down::<PAGE_SIZE>(fdt_ptr as usize),
        align_up::<PAGE_SIZE>(fdt_size),
    )
    .unwrap();
    unsafe {
        let fdt = Fdt::from_ptr(fdt_ptr as *const u8).unwrap();
        globals::initialize(fdt);
    };

    uart_driver.send_str("> initializing mmu\n");
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
    frame::set_kernel_trap_frame(HartId::zero());

    // Initialize drivers
    DRIVERS.init(&globals::get().device_tree).unwrap();

    unsafe {
        riscv::register::sstatus::set_spie();
        riscv::register::sstatus::set_sum();
        // Timer interrupts are triggered using ssoft instead of stimer because we can clear ssoft
        // from supervisor mode
        riscv::register::sie::set_ssoft();
        riscv::register::sie::set_sext();
    }

    // Initialize timer
    timer::set_timer_period(HartId::zero(), Duration::from_millis(100)).unwrap();

    println!("{}", "Welcome to KabutOS!!!".cyan().bold());

    #[cfg(feature = "test")]
    krabby::test::test_and_exit();

    loop {
        run_console();
    }
}
