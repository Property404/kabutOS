#![no_std]
#![no_main]
use krabby::drivers::{ns16550::Ns16550Driver, UartDriver, DRIVERS};

#[no_mangle]
pub fn kmain() {
    // Initialize drivers
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    uart_driver.send_str("Welcome to KabutOS\n");
    unsafe { DRIVERS.uart = Some(uart_driver) };

    loop {
        unsafe {
            krabby::c_functions::run_console();
        }
    }
}
