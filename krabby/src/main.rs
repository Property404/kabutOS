#![no_std]
#![no_main]
use core::fmt::Write;
use krabby::{
    ansi_codes::CLEAR_SCREEN,
    drivers::{ns16550::Ns16550Driver, DRIVERS},
    serial::Serial,
};
use owo_colors::OwoColorize;

#[no_mangle]
unsafe fn kmain(_hart_id: usize, fdt_ptr: *const u8) {
    // Initialize drivers
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    unsafe { DRIVERS.uart = Some(uart_driver) };

    let mut serial = Serial::new();
    writeln!(
        serial,
        "{CLEAR_SCREEN}{}",
        "Welcome to KabutOS!!!".cyan().bold()
    )
    .unwrap();
    writeln!(serial, "Device tree is @ {fdt_ptr:p}").unwrap();

    loop {
        unsafe {
            krabby::c_functions::run_console();
        }
    }
}
