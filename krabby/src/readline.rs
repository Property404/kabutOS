//! GNU Readline-like functionality
use crate::{ansi_codes::CLEAR_LINE, errors::KernelResult, serial::Serial};
use core::{fmt::Write, str};
use utf8_parser::ParsedByte;

const DELETE: char = '\x7F';
const BACKSPACE: char = '\x08';

/// Read line of user input
pub fn get_line<'a>(prompt: &str, buffer: &'a mut [u8]) -> KernelResult<&'a str> {
    let mut serial = Serial::new();

    // This points to the current *byte* in the array, which may or may be different from the
    // current *character* since *characters* are variable length
    let mut byte_ptr = 0;

    write!(serial, "{prompt}")?;
    loop {
        match serial.next_char()? {
            // <Enter> is pressed. Print a new line and return
            '\r' => {
                writeln!(serial)?;
                return Ok(core::str::from_utf8(&buffer[0..byte_ptr])?);
            }

            // <BackSpace> is pressed. Delete last character.
            DELETE | BACKSPACE => {
                // Rewind to UTF-8 start byte
                while byte_ptr > 0 && ParsedByte::try_from(buffer[byte_ptr - 1])?.is_continuation()
                {
                    byte_ptr -= 1;
                }
                // Delete start byte
                if byte_ptr > 0 {
                    byte_ptr -= 1;
                }
            }

            // Ignore all other control characters
            // Explicitly continue so we don't redraw
            c if c.is_ascii_control() => continue,

            // Character is entered - Echo and place on buffer
            c => {
                if byte_ptr + c.len_utf8() < buffer.len() {
                    c.encode_utf8(&mut buffer[byte_ptr..]);
                    byte_ptr += c.len_utf8();
                }
            }
        }

        write!(
            serial,
            "{CLEAR_LINE}\r{prompt}{}",
            str::from_utf8(&buffer[0..byte_ptr])?
        )?;
    }
}
