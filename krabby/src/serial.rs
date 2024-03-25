//! Home of the `Serial` object - used to read from serial
use crate::{
    drivers::{Driver, UartDriver, DRIVERS},
    KernelError, KernelResult,
};
use alloc::sync::Arc;
use spin::Mutex;

/// A structure used to read from serial.
#[derive(Clone, Debug)]
pub struct Serial(Arc<Mutex<Driver<dyn UartDriver>>>);

impl Serial {
    /// Construct a new `Serial` object
    pub fn new() -> KernelResult<Self> {
        let uart = DRIVERS.uart.read();
        if let Some(uart) = &*uart {
            Ok(Self(uart.clone()))
        } else {
            Err(KernelError::DriverUninitialized)
        }
    }

    /// Read next character
    pub fn next_char(&self) -> KernelResult<char> {
        Ok(self.0.lock().coupling.spin_until_next_char())
    }
}

// Macros copied from <https://osblog.stephenmarz.com/ch2.html>

#[macro_export]
macro_rules! print
{
    ($($args:expr),+) => ({
            $crate::print!([_inner (false)] $($args),+);
    });
    ([_inner ($newline:expr)] $($args:expr),+) => ({
            use core::fmt::Write;
            let uart = $crate::drivers::DRIVERS.uart.read();
            if let Some(uart) = &*uart{
                let mut uart = uart.lock();
                let _ = write!(uart.coupling, $($args),+);
                if $newline {
                    let _ = uart.coupling.write_str("\n");
                }
            }
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        $crate::print!("\n")
    });
    ($($args:expr),+) => ({
        $crate::print!([_inner (true)] $($args),+);
    });
}
