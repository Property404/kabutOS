//! Drivers and driver accessories
use crate::KernelResult;
use alloc::boxed::Box;
use core::fmt::Debug;
use fdt::{node::FdtNode, Fdt};
pub mod ns16550;
pub mod sifive_uart;
use utf8_parser::Utf8Parser;

/// Collection of initialized drivers
#[derive(Debug)]
pub struct Drivers {
    /// The UART driver
    pub uart: Option<Box<dyn UartDriver>>,
}

impl Drivers {
    fn init_uart(&mut self, node: &FdtNode) -> KernelResult<()> {
        // Don't reinit
        if self.uart.is_some() {
            return Ok(());
        }

        self.uart = ns16550::Ns16550Driver::maybe_from_node(node)
            .transpose()
            .or_else(|| sifive_uart::SifiveUartDriver::maybe_from_node(node).transpose())
            .transpose()?;

        Ok(())
    }

    pub fn init(&mut self, fdt: &Fdt) -> KernelResult<()> {
        let chosen = fdt.chosen();
        if let Some(stdout) = chosen.stdout() {
            self.init_uart(&stdout.node())?;
        }

        Ok(())
    }
}

/// Global object that keeps track of initialized drivers
pub static mut DRIVERS: Drivers = Drivers { uart: None };

/// Driver for a "disk.' This can be NOR flash, an SSD, a hard drive, or just RAM.
pub trait DiskDriver: Debug {
    /// Read one byte from `address`
    fn read8(&self, address: usize) -> KernelResult<u8>;
    /// Write one byte to `address`
    fn write8(&self, address: usize, value: u8) -> KernelResult<()>;
}

/// A UART/serial driver
pub trait UartDriver: Debug {
    /// Read the next byte out of the UART
    fn next_byte(&self) -> u8;

    /// Write a byte to the UART
    fn send_byte(&self, byte: u8);

    /// Read the next character from the UART
    fn next_char(&self) -> char {
        let mut parser = Utf8Parser::default();
        loop {
            if let Some(c) = parser
                .push(self.next_byte())
                .unwrap_or(Some(char::REPLACEMENT_CHARACTER))
            {
                return c;
            }
        }
    }

    /// Send a string to the UART
    fn send_str(&self, s: &str) {
        for byte in s.as_bytes() {
            self.send_byte(*byte)
        }
    }
}
