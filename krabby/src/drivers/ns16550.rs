//! Ns16550 Driver
use crate::c_functions::{read_unaligned_volatile_u8, write_unaligned_volatile_u8};
use crate::drivers::UartDriver;

enum RegisterOffsets {
    Data = 0x00,
    InterruptEnable = 0x01,
    FifoControl = 0x02,
    LineControlRegister = 0x03,
    LineStatusRegister = 0x05,
}

/// UART driver for the [Ns16550](https://en.wikipedia.org/wiki/NS16550A)
///
/// See manual here: <https://uart16550.readthedocs.io/_/downloads/en/latest/pdf/>
#[derive(Debug)]
pub struct Ns16550Driver {
    base_address: *mut u8,
}
unsafe impl Sync for Ns16550Driver {}

impl Ns16550Driver {
    /// Initialize the driver
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
        unsafe { read_unaligned_volatile_u8(self.base_address.wrapping_add(offset as usize)) }
    }
}

impl UartDriver for Ns16550Driver {
    fn next_byte(&self) -> u8 {
        while !self.byte_available() {}
        unsafe { self.read(RegisterOffsets::Data) }
    }

    fn byte_available(&self) -> bool {
        unsafe { self.read(RegisterOffsets::LineStatusRegister) & 0x01 != 0 }
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
