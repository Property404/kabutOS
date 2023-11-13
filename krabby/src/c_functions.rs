use crate::drivers::{UartDriver, DRIVERS};
use core::ffi::{c_char, c_int};

extern "C" {
    pub fn read_unaligned_volatile_u8(_: *const u8) -> u8;
    pub fn write_unaligned_volatile_u8(_: *mut u8, _: u8);
    pub fn run_console();
}

#[no_mangle]
pub fn putchar(c: c_char) -> c_int {
    unsafe {
        if let Some(uart) = &DRIVERS.uart {
            uart.send_byte(c as u8);
        }
    }
    0
}

#[no_mangle]
pub fn getchar() -> c_char {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.next_byte() as c_char;
    }
    // Send a question mark
    0x3F as c_char
}

#[no_mangle]
pub fn char_available() -> bool {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.byte_available();
    }
    false
}
