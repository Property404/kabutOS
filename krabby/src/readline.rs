//! GNU Readline-like functionality
use crate::errors::KernelResult;
use crate::serial::Serial;
use core::fmt::Write;

/// Read line of user input
pub fn get_line(buffer: &mut [u8]) -> KernelResult<&str> {
    let mut serial = Serial::new();
    let mut byte_ptr = 0;

    loop {
        let c = serial.next_char()?;
        if c == '\r' {
            let _ = writeln!(serial);
            return Ok(core::str::from_utf8(buffer).unwrap());
        }
        if byte_ptr + c.len_utf8() < buffer.len() && !c.is_ascii_control() {
            if !c.is_ascii_control() {
                let _ = write!(serial, "{c}");
                c.encode_utf8(&mut buffer[byte_ptr..]);
            }
            byte_ptr += c.len_utf8();
        }
    }
}
