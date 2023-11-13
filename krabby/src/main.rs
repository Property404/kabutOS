#![no_std]
#![no_main]
use krabby::{driver::UartDriver, uart::Ns16550Driver};

#[no_mangle]
pub fn kmain() {
    let uart_driver = Ns16550Driver::new(0x10000000 as *mut u8);
    uart_driver.send_str("Welcome to KabutOS\n");
    loop {
        unsafe {
            krabby::helpers::run_console();
        }
    }
}
