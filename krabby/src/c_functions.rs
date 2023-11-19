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

/// C wrapper of readline functionality
/// Note that C will not handle unicode characters correctly, so this is quite unsafe
///
/// # Safety
/// `array` must be mutable memory, and `max_size` must be accurate
/// The caller must handle unicode correctly (which it won't, lol)
#[no_mangle]
pub unsafe fn readline(array: *mut u8, max_size: usize) -> usize {
    let bytes: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(array, max_size) };
    let val = crate::readline::get_line("KabutOS➔  ", bytes).unwrap();
    let size = val.len();
    bytes[size] = 0;
    size
}
