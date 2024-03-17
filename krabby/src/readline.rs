//! GNU Readline-like functionality
use crate::{
    ansi_codes::{CLEAR_LINE, CLEAR_SCREEN},
    print, println,
    serial::Serial,
    KernelResult,
};
use core::str;
use embedded_line_edit::{LineEditBufferWithHistoryRing, LineEditState};
use owo_colors::OwoColorize;

const DELETE: char = '\x7F';
const BACKSPACE: char = '\x08';
const ESCAPE: char = '\x1b';
const CONTROL_A: char = '\x01';
const CONTROL_B: char = '\x02';
const CONTROL_C: char = '\x03';
const CONTROL_D: char = '\x04';
const CONTROL_E: char = '\x05';
const CONTROL_F: char = '\x06';
const CONTROL_K: char = '\x0b';
const CONTROL_L: char = '\x0c';
const CONTROL_T: char = '\x14';
const CONTROL_W: char = '\x17';

/// Readline object used to retrieve user input
pub struct Readline<const BUFFER_SIZE: usize, const HISTORY_SIZE: usize> {
    buffer: LineEditState<LineEditBufferWithHistoryRing<[u8; BUFFER_SIZE], HISTORY_SIZE>>,
}

impl<const BUFFER_SIZE: usize, const HISTORY_SIZE: usize> Default
    for Readline<BUFFER_SIZE, HISTORY_SIZE>
{
    fn default() -> Self {
        let buffer_with_history = LineEditBufferWithHistoryRing::from_buffer([0; BUFFER_SIZE]);
        let buffer = LineEditState::from_buffer(buffer_with_history);
        Self { buffer }
    }
}

impl<const BUFFER_SIZE: usize, const HISTORY_SIZE: usize> Readline<BUFFER_SIZE, HISTORY_SIZE> {
    /// Read line of user input
    pub fn get_line<'a>(&'a mut self, prompt: &str) -> KernelResult<&'a str> {
        let serial = Serial::new();
        if !self.buffer.is_empty() {
            self.buffer.new_history_entry();
        }

        let prompt = prompt.cyan();
        print!("{prompt}");
        loop {
            // For optimization purposes
            // Set to true if we're only moving the cursor
            let mut shift_only = false;

            match serial.next_char()? {
                // <Enter> is pressed. Print a new line and return
                '\r' => {
                    println!();
                    return Ok(self.buffer.as_str()?);
                }

                // <BackSpace> is pressed. Delete last character.
                DELETE | BACKSPACE => {
                    self.buffer.delete_prev()?;
                }

                CONTROL_A => {
                    self.buffer.move_to_start();
                    shift_only = true;
                }

                CONTROL_B => {
                    self.buffer.shift_left(1)?;
                    shift_only = true;
                }

                // Cancel
                CONTROL_C => {
                    println!();
                    return Ok("");
                }

                CONTROL_D => {
                    self.buffer.delete_current()?;
                }

                CONTROL_E => {
                    self.buffer.move_to_end();
                    shift_only = true;
                }

                CONTROL_F => {
                    self.buffer.shift_right(1)?;
                    shift_only = true;
                }

                CONTROL_K => {
                    self.buffer.kill_to_end()?;
                }

                CONTROL_L => {
                    print!("{CLEAR_SCREEN}");
                }

                CONTROL_T => {
                    self.buffer.transpose_chars()?;
                }

                CONTROL_W => {
                    self.buffer.kill_prev_word()?;
                }

                // Arrow keys
                ESCAPE => {
                    match serial.next_char()? {
                        '[' => {
                            match serial.next_char()? {
                                // Left arrow key: <ESC>[D
                                'D' => {
                                    self.buffer.shift_left(1)?;
                                    shift_only = true;
                                }
                                // Right arrow key: <ESC>[C
                                'C' => {
                                    self.buffer.shift_right(1)?;
                                    shift_only = true;
                                }
                                // Up arrow key: <ESC>[A
                                'A' => {
                                    self.buffer.prev_history_entry();
                                }
                                // Down arrow key: <ESC>[B
                                'B' => {
                                    self.buffer.next_history_entry();
                                }
                                _ => continue,
                            };
                        }
                        // Alt-b: <ESC>b
                        'b' => {
                            self.buffer.move_to_prev_start_of_word()?;
                            shift_only = true;
                        }
                        // Alt-f: <ESC>f
                        'f' => {
                            self.buffer.move_past_end_of_word()?;
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
                    if self.buffer.insert(c) {
                        // This should be a bit more efficient than a complete redraw
                        let tail = self.buffer.tail()?;
                        print!("{c}{tail}");
                        for _ in 0..tail.len() {
                            print!("\x08");
                        }
                    }
                    continue;
                }
            }

            if shift_only {
                // Just place the cursor correctly
                print!("\r{prompt}{}", self.buffer.head()?);
            } else {
                print!(
                    "{CLEAR_LINE}\r{prompt}{}\r{prompt}{}",
                    // Write whole line
                    self.buffer.as_str()?,
                    // Then place cursor at `byte_ptr`
                    self.buffer.head()?
                );
            }
        }
    }
}
