#![no_std]
#![no_main]
use core::fmt::Write;
use krabby::{
    drivers::{ns16550::Ns16550Driver, DRIVERS},
    serial::Serial,
};

#[no_mangle]
unsafe fn kmain() {
    // Initialize drivers
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    unsafe { DRIVERS.uart = Some(uart_driver) };

    let mut serial = Serial::new();
    writeln!(serial, "Welcome to KabutOS!!!").unwrap();

    loop {
        unsafe {
            krabby::c_functions::run_console();
        }
    }
}
