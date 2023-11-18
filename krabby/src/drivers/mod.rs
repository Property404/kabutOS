//! Drivers and driver accessories
use crate::errors::KernelResult;
pub mod ns16550;
use ns16550::Ns16550Driver;
use utf8_parser::Utf8Parser;

/// Collection of initialized drivers
#[derive(Debug)]
pub struct Drivers {
    /// The UART driver
    // TODO: make this dynamic
    pub uart: Option<Ns16550Driver>,
}

/// Global object that keeps track of initialized drivers
pub static mut DRIVERS: Drivers = Drivers { uart: None };

/// Generic driver supertrait
pub trait Driver: Sized {}

/// Driver for a "disk.' This can be NOR flash, an SSD, a hard drive, or just RAM.
pub trait DiskDriver: Driver {
    /// Read one byte from `address`
    fn read8(&self, address: usize) -> KernelResult<u8>;
    /// Write one byte to `address`
    fn write8(&self, address: usize, value: u8) -> KernelResult<()>;
}

/// A UART/serial driver
pub trait UartDriver: Driver {
    /// Read the next byte out of the UART
    fn next_byte(&self) -> u8;

    /// Write a byte to the UART
    fn send_byte(&self, byte: u8);

    /// Check if a byte is available to be read
    fn byte_available(&self) -> bool;

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
        for c in s.chars() {
            // Rust works in UTF-8 (unlike C, which generally works in ASCII), so we have to convert a Rust
            // `char` to a sequence of c `chars`(which are just bytes)
            let mut bytes = [0; 4];
            for byte in c.encode_utf8(&mut bytes).as_bytes() {
                self.send_byte(*byte)
            }
        }
    }
}
