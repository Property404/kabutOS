//! GNU Readline-like functionality
use crate::errors::KernelResult;
use crate::serial::Serial;
use core::{fmt::Write, str};

/// Read line of user input
pub fn get_line(buffer: &mut [u8]) -> KernelResult<&str> {
    let mut serial = Serial::new();

    // This points to the current *byte* in the array, which may or may be different from the
    // current *character* since *characters* are variable length
    let mut byte_ptr = 0;

    loop {
        match serial.next_char()? {
            // <Enter> is pressed. Print a new line and return
            '\r' => {
                writeln!(serial)?;
                return Ok(core::str::from_utf8(buffer)?);
            }

            // Ignore all other control characters
            c if c.is_ascii_control() => {}

            // Character is entered - Echo and place on buffer
            c => {
                if byte_ptr + c.len_utf8() < buffer.len() {
                    write!(serial, "{c}")?;
                    c.encode_utf8(&mut buffer[byte_ptr..]);
                    byte_ptr += c.len_utf8();
                }
            }
        }
    }
}
