//! GNU Readline-like functionality
use crate::{
    ansi_codes::{CLEAR_LINE, CLEAR_SCREEN},
    errors::KernelResult,
    serial::Serial,
};
use core::{fmt::Write, str};
use embedded_line_edit::LineEditState;
use owo_colors::OwoColorize;

const DELETE: char = '\x7F';
const BACKSPACE: char = '\x08';
const ESCAPE: char = '\x1b';
const CONTROL_A: char = '\x01';
const CONTROL_B: char = '\x02';
const CONTROL_D: char = '\x04';
const CONTROL_E: char = '\x05';
const CONTROL_F: char = '\x06';
const CONTROL_L: char = '\x0c';

/// Read line of user input
pub fn get_line<'a>(prompt: &str, buffer: &'a mut [u8]) -> KernelResult<&'a str> {
    let mut serial = Serial::new();
    let mut buffer = LineEditState::from_buffer(buffer);

    let prompt = prompt.cyan();
    let prompt = prompt.bold();
    write!(serial, "{prompt}")?;
    loop {
        // For optimization purposes
        // Set to true if we're only moving the cursor
        let mut shift_only = false;

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
                shift_only = true;
            }

            CONTROL_B => {
                buffer.shift_left(1)?;
                shift_only = true;
            }

            CONTROL_D => {
                buffer.delete_current()?;
            }

            CONTROL_E => {
                buffer.move_to_end();
                shift_only = true;
            }

            CONTROL_F => {
                buffer.shift_right(1)?;
                shift_only = true;
            }

            CONTROL_L => {
                write!(serial, "{CLEAR_SCREEN}")?;
            }

            // Arrow keys
            ESCAPE => {
                match serial.next_char()? {
                    '[' => {
                        match serial.next_char()? {
                            // Left arrow key: <ESC>[D
                            'D' => {
                                buffer.shift_left(1)?;
                                shift_only = true;
                            }
                            // Right arrow key: <ESC>[C
                            'C' => {
                                buffer.shift_right(1)?;
                                shift_only = true;
                            }
                            _ => continue,
                        };
                    }
                    // Alt-b: <ESC>b
                    'b' => {
                        buffer.move_to_prev_start_of_word()?;
                        shift_only = true;
                    }
                    // Alt-f: <ESC>f
                    'f' => {
                        buffer.move_past_end_of_word()?;
                        shift_only = true;
                    }
                    _ => {
                        continue;
                    }
                }
            }

            // Ignore all other control characters
            // Explicitly continue so we don't redraw
            c if c.is_ascii_control() => {
                continue;
            }

            // Character is entered - Echo and place on buffer
            c => {
                if buffer.insert(c) {
                    // This should be a bit more efficient than a complete redraw
                    let tail = buffer.tail()?;
                    write!(serial, "{c}{tail}")?;
                    for _ in 0..tail.len() {
                        write!(serial, "\x08")?;
                    }
                }
                continue;
            }
        }

        if shift_only {
            // Just place the cursor correctly
            write!(serial, "\r{prompt}{}", buffer.head()?)?;
        } else {
            write!(
                serial,
                "{CLEAR_LINE}\r{prompt}{}\r{prompt}{}",
                // Write whole line
                buffer.as_str()?,
                // Then place cursor at `byte_ptr`
                buffer.head()?
            )?;
        }
    }
}
