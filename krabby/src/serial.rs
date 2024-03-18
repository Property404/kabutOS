//! Home of the `Serial` object - used to read from serial
use crate::{drivers::DRIVERS, KernelError, KernelResult};

/// A cheap structure used to read from serial.
#[derive(Copy, Clone, Default, Debug)]
pub struct Serial {}

impl Serial {
    /// Construct a new `Serial` object. Very cheap.
    pub const fn new() -> Self {
        Self {}
    }

    /// Read next character
    pub fn next_char(&self) -> KernelResult<char> {
        let mut uart = DRIVERS.uart.lock();
        if let Some(uart) = &mut *uart {
            Ok(uart.next_char())
        } else {
            Err(KernelError::DriverUninitialized)
        }
    }
}

// Macros copied from <https://osblog.stephenmarz.com/ch2.html>

#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
            use core::fmt::Write;
            let mut uart = $crate::drivers::DRIVERS.uart.lock();
            if let Some(uart) = &mut *uart {
                let _ = write!(uart, $($args)+);
            }
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        print!("\n")
    });
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let mut uart = $crate::drivers::DRIVERS.uart.lock();
        if let Some(uart) = &mut *uart {
            let _ = write!(uart, $($args)+);
            let _ = write!(uart, "\n");
        }
    });
}
