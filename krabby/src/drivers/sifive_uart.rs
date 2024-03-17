//! Uart driver for SiFive boards
//!
//! Partially adapted from
//!     <https://github.com/riscv-software-src/opensbi/blob/master/lib/utils/serial/sifive-uart.c>
//!     Copyright (c) 2019 Western Digital Corporation or its affiliates, under the BSD license.
//!     Author: Anup Patel
use crate::{
    drivers::UartDriver,
    mmu::{map_device, PAGE_SIZE},
    KernelError, KernelResult,
};
use alloc::boxed::Box;
use bilge::prelude::*;
use core::ptr::{self, read_volatile, write_volatile};
use fdt::node::FdtNode;

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

unsafe impl Send for SifiveUartDriver {}

impl SifiveUartDriver {
    const COMPATIBLE_STRING: &'static str = "sifive,uart0";

    /// Initialize the driver
    pub fn maybe_from_node(node: &FdtNode) -> KernelResult<Option<Box<dyn UartDriver>>> {
        let Some(compatible) = node.compatible() else {
            return Ok(None);
        };
        if compatible.first() != Self::COMPATIBLE_STRING {
            return Ok(None);
        }

        let reg = node
            .reg()
            .and_then(|mut v| v.next())
            .ok_or(KernelError::MissingProperty("reg"))?;
        let base_address = map_device(reg.starting_address as usize, PAGE_SIZE)?;

        Ok(Some(Box::new(Self::new(base_address as *mut u8))))
    }

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

impl UartDriver for SifiveUartDriver {
    fn next_byte(&mut self) -> u8 {
        loop {
            let rx_fifo = unsafe { (read_volatile(self.uart)).rx_fifo };
            if !rx_fifo.empty() {
                return rx_fifo.data();
            }
        }
    }

    fn send_byte(&mut self, byte: u8) {
        unsafe {
            write_volatile(ptr::from_mut(&mut (*self.uart).tx_fifo), byte as u32);
        }
    }
}
