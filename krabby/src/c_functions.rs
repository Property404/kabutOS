//! C function imports and exports
use crate::drivers::{UartDriver, DRIVERS};
use core::ffi::{c_char, c_int};

extern "C" {
    /// Read one byte from an unaligned address
    pub fn read_unaligned_volatile_u8(ptr: *const u8) -> u8;
    /// Write one byte to an unaligned address
    pub fn write_unaligned_volatile_u8(ptr: *mut u8, _: u8);
    /// Run the kernel console
    pub fn run_console();
}

#[no_mangle]
/// Kernel equivalent of `putchar`(3)
/// Pull a character from serial
pub fn putchar(c: c_char) -> c_int {
    unsafe {
        if let Some(uart) = &DRIVERS.uart {
            uart.send_byte(c as u8);
        }
    }
    0
}

#[no_mangle]
/// Kernel equivalent of `getchar`(3)
/// Send a character to serial
/// Please make sure a character is actually available by checking [char_available]
pub fn getchar() -> c_char {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.next_byte() as c_char;
    }
    // Send a question mark - what else would we do if the uart is not initialized?
    0x3F as c_char
}

#[no_mangle]
/// Check if a character is currently available to be read from serial
pub fn char_available() -> bool {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.byte_available();
    }
    false
}
