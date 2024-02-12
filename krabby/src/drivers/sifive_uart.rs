//! Uart driver for SiFive boards
//!
//! Partially adapted from
//!     <https://github.com/riscv-software-src/opensbi/blob/master/lib/utils/serial/sifive-uart.c>
//!     Copyright (c) 2019 Western Digital Corporation or its affiliates, under the BSD license.
//!     Author: Anup Patel
use crate::drivers::{Driver, UartDriver};
use bilge::prelude::*;
use core::ptr::{self, read_volatile, write_volatile};

// Memory-mapped Sifive Uart
#[repr(C, packed(32))]
#[derive(Debug)]
struct Uart {
    tx_fifo: u32,
    rx_fifo: RxFifo,
    tx_ctrl: u32,
    rx_ctrl: u32,
    ie: u32,
    ip: u32,
    div: u32,
}

#[bitsize(32)]
#[derive(Copy, Clone, DebugBits)]
struct RxFifo {
    data: u8,
    _unused: u23,
    empty: bool,
}

/// Sifive Uart driver
#[derive(Debug)]
pub struct SifiveUartDriver {
    uart: *mut Uart,
}

unsafe impl Sync for SifiveUartDriver {}

impl SifiveUartDriver {
    /// Initialize the driver
    pub fn new(base_address: *mut u8) -> Self {
        let uart = base_address as *mut Uart;

        // TODO: baudrate needs to be set on real hardware
        unsafe {
            // Disable interrupts
            write_volatile(ptr::from_mut(&mut (*uart).ie), 0);

            // Enable TX/RX
            write_volatile(ptr::from_mut(&mut (*uart).tx_ctrl), 1);
            write_volatile(ptr::from_mut(&mut (*uart).rx_ctrl), 1);
        }

        Self { uart }
    }
}

impl Driver for SifiveUartDriver {}

impl UartDriver for SifiveUartDriver {
    fn next_byte(&self) -> u8 {
        loop {
            let rx_fifo = unsafe { (read_volatile(self.uart)).rx_fifo };
            if !rx_fifo.empty() {
                return rx_fifo.data();
            }
        }
    }

    fn send_byte(&self, byte: u8) {
        unsafe {
            write_volatile(ptr::from_mut(&mut (*self.uart).tx_fifo), byte as u32);
        }
    }
}
