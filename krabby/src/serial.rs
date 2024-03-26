//! Home of the `Serial` object - used to read from serial
use crate::{
    drivers::{Driver, UartDriver, DRIVERS},
    KernelError, KernelResult,
};
use alloc::sync::Arc;
use core::{fmt, iter};
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
}

impl iter::Iterator for Serial {
    type Item = KernelResult<char>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(self.0.lock().coupling.spin_until_next_char()))
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.0.lock().coupling.send_str(s);
        Ok(())
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

            if let Ok(mut serial) = $crate::serial::Serial::new() {
                let _ = write!(serial, $($args),+);
                if $newline {
                    let _ = serial.write_str("\n");
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

#[macro_export]
macro_rules! warn
{
    ($($args:expr),+) => ({
        use owo_colors::OwoColorize;
        $crate::print!("{}","[warning] ".yellow());
        $crate::println!($($args),+);
    });
}
