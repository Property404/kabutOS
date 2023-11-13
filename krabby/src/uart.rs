use crate::driver::UartDriver;
use crate::helpers::{read_unaligned_volatile_u8, write_unaligned_volatile_u8};
use core::ffi::{c_char, c_int};

#[repr(usize)]
enum RegisterOffsets {
    Data = 0x00,
    InterruptEnable = 0x01,
    FifoControl = 0x02,
    LineControlRegister = 0x03,
    LineStatusRegister = 0x05,
}

pub struct Ns16550Driver {
    base_address: *mut u8,
}

impl Ns16550Driver {
    pub fn new(base_address: *mut u8) -> Self {
        unsafe {
            write_unaligned_volatile_u8(
                base_address.wrapping_add(RegisterOffsets::LineControlRegister as usize),
                0x03,
            );
            write_unaligned_volatile_u8(
                base_address.wrapping_add(RegisterOffsets::FifoControl as usize),
                0x01,
            );
            write_unaligned_volatile_u8(
                base_address.wrapping_add(RegisterOffsets::InterruptEnable as usize),
                0x01,
            );
        }
        Self { base_address }
    }

    unsafe fn read(&self, offset: RegisterOffsets) -> u8 {
        read_unaligned_volatile_u8(self.base_address.wrapping_add(offset as usize))
    }
}

impl UartDriver for Ns16550Driver {
    fn next_byte(&self) -> u8 {
        while unsafe { self.read(RegisterOffsets::LineStatusRegister) & 0x01 == 0 } {}
        unsafe { self.read(RegisterOffsets::Data) }
    }

    fn send_byte(&self, byte: u8) {
        unsafe {
            write_unaligned_volatile_u8(
                self.base_address
                    .wrapping_add(RegisterOffsets::Data as usize),
                byte,
            );
        }
    }
}

const DR_REGISTER: *mut u8 = 0x10000000 as *mut u8;
const LSR_REGISTER: *mut u8 = 0x10000005 as *mut u8;

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
