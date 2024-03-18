//! Home of the `Serial` object - used to write to serial
use crate::{drivers::DRIVERS, KernelError, KernelResult};
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

    /// Returns true if driver loaded
    pub fn driver_loaded() -> bool {
        let uart = DRIVERS.uart.lock();
        uart.is_some()
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

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let mut uart = DRIVERS.uart.lock();
        if let Some(uart) = &mut *uart {
            uart.send_str(s);
        }
        Ok(())
    }
}

// Macros copied from <https://osblog.stephenmarz.com/ch2.html>

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
			use core::fmt::Write;
            use $crate::{serial::Serial};
            if Serial::driver_loaded() {
                let _ = write!(Serial::new(), $($args)+);
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
        use $crate::{serial::Serial};
        if Serial::driver_loaded() {
			let _ = write!(Serial::new(), $($args)+);
            let _ = write!(Serial::new(), "\n");
        }
	});
}
