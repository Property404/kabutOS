//! Ns16550 Driver
use crate::c_functions::{read_unaligned_volatile_u8, write_unaligned_volatile_u8};
use crate::drivers::{Driver, UartDriver};

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
        let driver = Self { base_address };

        unsafe {
            driver.write(RegisterOffsets::LineControlRegister, 0x03);
            driver.write(RegisterOffsets::FifoControl, 0x01);
            driver.write(RegisterOffsets::InterruptEnable, 0x01);
        }

        driver
    }

    unsafe fn write(&self, offset: RegisterOffsets, value: u8) {
        unsafe {
            write_unaligned_volatile_u8(self.base_address.wrapping_add(offset as usize), value)
        }
    }

    unsafe fn read(&self, offset: RegisterOffsets) -> u8 {
        unsafe { read_unaligned_volatile_u8(self.base_address.wrapping_add(offset as usize)) }
    }
}

impl Driver for Ns16550Driver {}

impl UartDriver for Ns16550Driver {
    fn next_byte(&self) -> u8 {
        // Wait until byte is available
        while unsafe { self.read(RegisterOffsets::LineStatusRegister) & 0x01 == 0 } {}
        // Then read the byte
        unsafe { self.read(RegisterOffsets::Data) }
    }

    fn send_byte(&self, byte: u8) {
        unsafe {
            self.write(RegisterOffsets::Data, byte);
        }
    }
}
