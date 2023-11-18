//! GNU Readline-like functionality
use crate::{ansi_codes::CLEAR_LINE, errors::KernelResult, serial::Serial};
use core::{fmt::Write, str};
use embedded_line_edit::LineEditState;

const DELETE: char = '\x7F';
const BACKSPACE: char = '\x08';
const ESCAPE: char = '\x1b';
const CONTROL_A: char = '\x01';
const CONTROL_B: char = '\x02';
const CONTROL_D: char = '\x04';
const CONTROL_E: char = '\x05';
const CONTROL_F: char = '\x06';

/// Read line of user input
pub fn get_line<'a>(prompt: &str, buffer: &'a mut [u8]) -> KernelResult<&'a str> {
    let mut serial = Serial::new();

    let mut buffer = LineEditState::from_buffer(buffer);

    write!(serial, "{prompt}")?;
    loop {
        match serial.next_char()? {
            // <Enter> is pressed. Print a new line and return
            '\r' => {
                writeln!(serial)?;
                return Ok(buffer.to_str()?);
            }

            // <BackSpace> is pressed. Delete last character.
            DELETE | BACKSPACE => {
                buffer.delete_prev()?;
            }

            CONTROL_A => {
                buffer.move_to_start();
            }

            CONTROL_B => {
                buffer.shift_left(1)?;
            }

            CONTROL_D => {
                buffer.delete_current()?;
            }

            CONTROL_E => {
                buffer.move_to_end();
            }

            CONTROL_F => {
                buffer.shift_right(1)?;
            }

            // Arrow keys
            ESCAPE => {
                match serial.next_char()? {
                    '[' => {
                        match serial.next_char()? {
                            // Left arrow key: <ESC>[D
                            'D' => {
                                buffer.shift_left(1)?;
                            }
                            // Right arrow key: <ESC>[C
                            'C' => {
                                buffer.shift_right(1)?;
                            }
                            _ => {}
                        };
                    }
                    // Alt-f: <ESC>f
                    'f' => {
                        // TODO: Make this go to next word
                        buffer.shift_right(1)?;
                    }
                    _ => {}
                }
            }

            // Ignore all other control characters
            // Explicitly continue so we don't redraw
            c if c.is_ascii_control() => {
                continue;
            }

            // Character is entered - Echo and place on buffer
            c => {
                buffer.insert(c);
            }
        }

        write!(
            serial,
            "{CLEAR_LINE}\r{prompt}{}\r{prompt}{}",
            // Write whole line
            buffer.as_str()?,
            // Then place cursor at `byte_ptr`
            buffer.as_partial_str()?
        )?;
    }
}
