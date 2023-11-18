//! C function imports and exports
use crate::{
    drivers::{UartDriver, DRIVERS},
    functions,
};
use core::ffi::{c_char, c_int};

extern "C" {
    /// Read one byte from an unaligned address
    pub fn read_unaligned_volatile_u8(ptr: *const u8) -> u8;
    /// Write one byte to an unaligned address
    pub fn write_unaligned_volatile_u8(ptr: *mut u8, _: u8);
    /// Run the kernel console
    pub fn run_console();
}

/// C API version of [functions::dump_memory]
///
/// # Safety
/// All addresses between `ptr` and `ptr+size`, inclusive, must be valid
#[no_mangle]
pub unsafe fn dump_memory(ptr: *const u8, size: usize) -> c_int {
    if unsafe { functions::dump_memory(ptr, size) }.is_err() {
        // Returning -1 is the standard way C returns errors
        -1
    } else {
        0
    }
}

/// Kernel equivalent of `putchar`(3)
/// Pull a character from serial
#[no_mangle]
pub fn putchar(c: c_char) -> c_int {
    unsafe {
        if let Some(uart) = &DRIVERS.uart {
            uart.send_byte(c as u8);
        }
    }
    0
}

/// Kernel equivalent of `getchar`(3)
/// Send a character to serial
/// Please make sure a character is actually available by checking [testchar]
#[no_mangle]
pub fn getchar() -> c_char {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.next_byte() as c_char;
    }
    // Send a question mark - what else would we do if the uart is not initialized?
    '?' as c_char
}

/// Check if a character is currently available to be read from serial
#[no_mangle]
pub fn testchar() -> bool {
    if let Some(uart) = unsafe { &DRIVERS.uart } {
        return uart.byte_available();
    }
    false
}

/// C wrapper of readline functionality
/// Note that C will not handle unicode characters correctly, so this is quite unsafe
///
/// # Safety
/// `array` must be mutable memory, and `max_size` must be accurate
/// The caller must handle unicode correctly (which it won't, lol)
#[no_mangle]
pub unsafe fn readline(array: *mut u8, max_size: usize) -> usize {
    let bytes: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(array, max_size) };
    let val = crate::readline::get_line("\x1b[35m>>>\x1b[0m ", bytes).unwrap();
    let size = val.len();
    bytes[size] = 0;
    size
}
