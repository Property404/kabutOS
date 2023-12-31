//! Home of the `Serial` object - used to write to serial
use crate::{
    drivers::{UartDriver, DRIVERS},
    KernelError, KernelResult,
};
use core::fmt::{Error, Write};

/// A cheap structure used to write to serial.
///
/// # Example
/// ```
/// use krabby::serial::Serial;
/// use core::fmt::Write;
///
/// writeln!(Serial::new(), "Hello World!");
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct Serial {}

impl Serial {
    /// Construct a new `Serial` object. Very cheap.
    pub const fn new() -> Self {
        Self {}
    }

    /// Read next character
    pub fn next_char(&self) -> KernelResult<char> {
        if let Some(uart) = unsafe { &DRIVERS.uart } {
            Ok(uart.next_char())
        } else {
            Err(KernelError::DriverUninitialized)
        }
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        if let Some(uart) = unsafe { &DRIVERS.uart } {
            uart.send_str(s);
        }
        Ok(())
    }
}
