//! Drivers and driver accessories
pub mod ns16550;
use ns16550::Ns16550Driver;

/// Collection of initialized drivers
pub struct Drivers {
    /// The UART driver
    // TODO: make this dynamic
    pub uart: Option<Ns16550Driver>,
}

/// Global object that keeps track of initialized drivers
pub static mut DRIVERS: Drivers = Drivers { uart: None };

/// A UART/serial driver
pub trait UartDriver {
    /// Read the next byte out of the UART
    fn next_byte(&self) -> u8;
    /// Write a byte to the UART
    fn send_byte(&self, byte: u8);
    /// Check if a byte is available to be read
    fn byte_available(&self) -> bool;
    /// Read the next character from the UART
    // Todo: Query next byte if incomplete UTF-8
    fn next_char(&self) -> char {
        self.next_byte() as char
    }
    /// Send a string to the UART
    fn send_str(&self, s: &str) {
        for c in s.chars() {
            // Rust works in UTF-8 (unlike C, which generally works in ASCII), so we have to convert a Rust
            // `char` to a sequence of c `chars`(which are just bytes)
            let mut bytes = [0; 4];
            c.encode_utf8(&mut bytes);
            for byte in &bytes[0..c.len_utf8()] {
                self.send_byte(*byte)
            }
        }
    }
}
