use crate::drivers::{UartDriver, DRIVERS};
use core::fmt::{Error, Write};

#[derive(Default)]
pub struct Serial {}

impl Serial {
    pub const fn new() -> Self {
        Self {}
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
