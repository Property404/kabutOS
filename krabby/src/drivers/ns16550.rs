//! Ns16550 Driver
use crate::{
    drivers::{DriverLoader, LoadInfo, LoadResult, UartDriver},
    mmu::{map_device, PAGE_SIZE},
    KernelError, KernelResult,
};
use alloc::boxed::Box;
use core::ptr::{read_volatile, write_volatile};

#[derive(Copy, Clone)]
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

unsafe impl Send for Ns16550Driver {}

impl Ns16550Driver {
    /// Initialize the driver
    pub fn new(base_address: *mut u8) -> Self {
        let mut driver = Self { base_address };

        unsafe {
            driver.write(RegisterOffsets::LineControlRegister, 0x03);
            driver.write(RegisterOffsets::FifoControl, 0x01);
            driver.write(RegisterOffsets::InterruptEnable, 0x01);
        }

        driver
    }

    unsafe fn write(&mut self, offset: RegisterOffsets, value: u8) {
        let address = self.base_address.wrapping_byte_add(offset as usize);
        unsafe { write_volatile(address, value) }
    }

    unsafe fn read(&self, offset: RegisterOffsets) -> u8 {
        let address = self.base_address.wrapping_byte_add(offset as usize);
        unsafe { read_volatile(address) }
    }
}

impl UartDriver for Ns16550Driver {
    fn next_byte(&mut self) -> u8 {
        // Wait until byte is available
        while unsafe { self.read(RegisterOffsets::LineStatusRegister) & 0x01 == 0 } {}
        // Then read the byte
        unsafe { self.read(RegisterOffsets::Data) }
    }

    fn send_byte(&mut self, byte: u8) {
        unsafe {
            self.write(RegisterOffsets::Data, byte);
        }
    }
}

fn load(info: &LoadInfo) -> KernelResult<LoadResult> {
    let reg = info
        .node
        .reg()
        .and_then(|mut v| v.next())
        .ok_or(KernelError::MissingProperty("reg"))?;
    let base_address = map_device(reg.starting_address as usize, PAGE_SIZE)?;

    Ok(LoadResult::Uart(Box::new(Ns16550Driver::new(
        base_address as *mut u8,
    ))))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "ns16550a",
    load,
};
