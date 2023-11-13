use crate::helpers::{read_unaligned_volatile_u8, write_unaligned_volatile_u8};
use core::ffi::{c_char, c_int};

const DR_REGISTER: *mut u8 = 0x10000000 as *mut u8;
const IER_REGISTER: *mut u8 = 0x10000001 as *mut u8;
const FIFO_REGISTER: *mut u8 = 0x10000002 as *mut u8;
const LCR_REGISTER: *mut u8 = 0x10000003 as *mut u8;
const LSR_REGISTER: *mut u8 = 0x10000005 as *mut u8;

#[no_mangle]
pub fn uart_init() {
    unsafe {
        write_unaligned_volatile_u8(LCR_REGISTER, 0x3);
        write_unaligned_volatile_u8(FIFO_REGISTER, 0x1);
        write_unaligned_volatile_u8(IER_REGISTER, 0x1);
    }
}

#[no_mangle]
pub fn putchar(c: c_char) -> c_int {
    unsafe {
        write_unaligned_volatile_u8(DR_REGISTER, c as u8);
    }
    0
}

#[no_mangle]
pub fn getchar() -> c_char {
    unsafe { read_unaligned_volatile_u8(DR_REGISTER) as c_char }
}

#[no_mangle]
pub fn char_available() -> bool {
    unsafe { read_unaligned_volatile_u8(LSR_REGISTER) & 0x01 != 0 }
}
