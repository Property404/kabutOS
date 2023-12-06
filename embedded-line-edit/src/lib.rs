//! No-allocation line editor core
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
use core::str::{self, Utf8Error};
use utf8_parser::{ParsedByte, Utf8Parser, Utf8ParserError};

/// Error type used for this crate
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LineEditError {
    /// Generic line editing error
    Generic(&'static str),
    /// Converted from [core::str::Utf8Error]
    Utf8(Utf8Error),
    /// Converted from [utf8_parser::Utf8ParserError]
    Utf8Parser(Utf8ParserError),
}

impl From<Utf8Error> for LineEditError {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<Utf8ParserError> for LineEditError {
    fn from(error: Utf8ParserError) -> Self {
        Self::Utf8Parser(error)
    }
}

/// A structure used to build a line editor
///
/// This structure does not come with any way to interpret key presses or allocate memory. You must
/// do all that yourself.
///
/// # Example
/// ```
/// use embedded_line_edit::LineEditState;
///
/// let mut bytes = [0u8;256];
/// let mut state = LineEditState::from_buffer(&mut bytes);
/// state.insert_many("Hello Worlf!".chars());
/// state.shift_left(1).unwrap();
/// state.delete_prev().unwrap();
/// state.insert('d');
/// state.shift_right(1).unwrap();
/// assert_eq!(state.as_str().unwrap(), "Hello World!");
/// ```
pub struct LineEditState<'a> {
    buffer: &'a mut [u8],
    byte_ptr: usize,
    byte_length: usize,
}

impl<'a> LineEditState<'a> {
    /// Construct a new [LineEditState]
    ///
    /// # Panics
    /// Panics if buffer is zero sized
    pub fn from_buffer(buffer: &'a mut [u8]) -> Self {
        assert!(!buffer.is_empty());
        Self {
            buffer,
            byte_ptr: 0,
            byte_length: 0,
        }
    }

    /// Get inner buffer as `&str`
    pub fn as_str(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(&self.buffer[0..self.byte_length])?)
    }

    /// Get inner buffer up to insertion point as `&str`
    pub fn head(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(&self.buffer[0..self.byte_ptr])?)
    }

    /// Get inner buffer from insertion point to end as `&str`
    pub fn tail(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(
            &self.buffer[self.byte_ptr..self.byte_length],
        )?)
    }

    /// Get inner buffer as `&str`
    pub fn to_str(self) -> Result<&'a str, LineEditError> {
        Ok(str::from_utf8(&self.buffer[0..self.byte_length])?)
    }

    /// Get current length of buffer in bytes.
    pub fn len(&self) -> usize {
        self.byte_length
    }

    /// Returns true if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.byte_length == 0
    }

    /// Clear whole buffer
    ///
    /// This is an O(1) operation
    pub fn clear(&mut self) {
        self.byte_ptr = 0;
        self.byte_length = 0;
    }

    /// Set insertion point to zero
    ///
    /// This is an O(1) operation
    pub fn move_to_start(&mut self) {
        self.byte_ptr = 0;
    }

    /// Set insertion point to end
    ///
    /// This is an O(1) operation
    pub fn move_to_end(&mut self) {
        self.byte_ptr = self.byte_length;
    }

    fn current_char(&self) -> Result<Option<char>, LineEditError> {
        let mut parser = Utf8Parser::default();
        let mut charlen = 0;
        debug_assert!(self.byte_ptr <= self.byte_length);
        if self.byte_ptr == self.byte_length {
            return Ok(None);
        }

        loop {
            if let Some(c) = parser.push(self.buffer[self.byte_ptr + charlen])? {
                return Ok(Some(c));
            }

            charlen += 1;
            if self.byte_ptr + charlen > self.byte_length {
                Err(LineEditError::Generic("UTF-8 character overrun"))?;
            }
        }
    }

    fn prev_char(&mut self) -> Result<Option<char>, LineEditError> {
        if self.shift_left(1)? == 0 {
            return Ok(None);
        }
        let c = self.current_char()?;
        self.shift_right(1)?;
        Ok(c)
    }

    /// Set insertion point to the last start-of-word
    /// This should be equivalent to alt+b in GNU Readline
    pub fn move_to_prev_start_of_word(&mut self) -> Result<(), LineEditError> {
        // We may be already at a start-of-word, so move back
        self.shift_left(1)?;

        // Cycle through whitespace
        loop {
            match self.current_char()? {
                Some(c) if c.is_whitespace() => {
                    self.shift_left(1)?;
                }
                Some(_) => {
                    break;
                }
                None => return Ok(()),
            }
        }

        // Cycle through non-whitespace
        while self.byte_ptr > 0 {
            match self.current_char()? {
                Some(c) if !c.is_whitespace() => {
                    self.shift_left(1)?;
                }
                Some(_) => {
                    // We moved past the start-of-word, move back and return
                    self.shift_right(1)?;
                    return Ok(());
                }
                // This should be unreachable
                None => return Ok(()),
            }
        }

        // First character
        Ok(())
    }

    /// Kill to the end of the line and return a reference to the killed portion
    /// This should be equivalent to ctrl+K in GNU Readline
    pub fn kill_to_end(&mut self) -> Result<&str, LineEditError> {
        let prev_length = self.byte_length;
        self.byte_length = self.byte_ptr;
        Ok(str::from_utf8(&self.buffer[self.byte_ptr..prev_length])?)
    }

    /// Kill the previous word and return a reference to it
    /// This should be equivalent to ctrl+w in GNU Readline
    pub fn kill_prev_word(&mut self) -> Result<&str, LineEditError> {
        let mut bytes_deleted = 0;

        let mut deleted_whitespace = false;
        loop {
            let Some(c) = self.prev_char()? else {
                break;
            };

            if c.is_whitespace() {
                if deleted_whitespace {
                    break;
                }
            } else {
                deleted_whitespace = true;
            }

            // Delete it
            assert_eq!(
                self.delete_prev()?
                    .expect("BUG: Found char but could not delete"),
                c
            );

            // Place at end of buffer so we can return it
            let clen = c.len_utf8();
            debug_assert!(clen > 0);
            let index = self.buffer.len() - bytes_deleted - clen;
            c.encode_utf8(&mut self.buffer[index..]);
            bytes_deleted += clen;
        }

        Ok(str::from_utf8(
            &self.buffer[self.buffer.len() - bytes_deleted..],
        )?)
    }

    /// Set insertion point past the end of the current word
    /// (or next word if not on a word)
    /// This should be equivalent to alt+f in GNU Readline
    pub fn move_past_end_of_word(&mut self) -> Result<(), LineEditError> {
        // Move to start of next word
        loop {
            match self.current_char()? {
                Some(c) if c.is_whitespace() => {
                    self.shift_right(1)?;
                }
                Some(_) => {
                    break;
                }
                None => return Ok(()),
            }
        }

        // Move to end of current word
        loop {
            match self.current_char()? {
                Some(c) if c.is_whitespace() => return Ok(()),
                Some(_) => {
                    self.shift_right(1)?;
                }
                None => return Ok(()),
            }
        }
    }

    /// Shift insertion point left by `n` characters
    /// This is an O(n) operation
    ///
    /// Returns number of characters shifted
    pub fn shift_left(&mut self, n: usize) -> Result<usize, LineEditError> {
        debug_assert!(self.buffer.len() >= self.byte_length);
        debug_assert!(self.byte_length >= self.byte_ptr);

        let mut shifted_by = 0;
        for _ in 0..n {
            // Rewind to UTF-8 start byte
            while self.byte_ptr > 0
                && ParsedByte::try_from(self.buffer[self.byte_ptr - 1])?.is_continuation()
            {
                self.byte_ptr -= 1;
            }
            if self.byte_ptr > 0 {
                self.byte_ptr -= 1;
            } else {
                break;
            }
            shifted_by += 1;
        }
        Ok(shifted_by)
    }

    /// Shift insertion point right by `n` characters
    /// This is an O(n) operation
    ///
    /// Returns number of characters shifted
    pub fn shift_right(&mut self, n: usize) -> Result<usize, LineEditError> {
        debug_assert!(self.buffer.len() >= self.byte_length);
        debug_assert!(self.byte_length >= self.byte_ptr);

        let mut shifted_by = 0;
        for _ in 0..n {
            if self.byte_ptr < self.byte_length {
                self.byte_ptr += 1;
            } else {
                break;
            };
            // Forward to next UTF-8 start byte
            while self.byte_ptr < self.byte_length
                && ParsedByte::try_from(self.buffer[self.byte_ptr])?.is_continuation()
            {
                self.byte_ptr += 1;
            }
            shifted_by += 1;
        }
        Ok(shifted_by)
    }

    /// Insert multiple
    ///
    /// Returns number of characters inserted
    pub fn insert_many(&mut self, chars: impl IntoIterator<Item = char>) -> usize {
        let mut count = 0;
        for c in chars.into_iter() {
            if !self.insert(c) {
                break;
            }
            count += 1;
        }
        count
    }

    /// Insert a character
    ///
    /// Returns `true` if inserted, or `false` if the buffer is full
    pub fn insert(&mut self, c: char) -> bool {
        debug_assert!(self.buffer.len() >= self.byte_length);
        debug_assert!(self.byte_length >= self.byte_ptr);

        let charlen = c.len_utf8();

        if self.byte_length + charlen > self.buffer.len() {
            return false;
        }

        // First shift everything right by the character length
        for i in ((self.byte_ptr)..(self.byte_length)).rev() {
            self.buffer[i + charlen] = self.buffer[i]
        }

        // Then copy the character onto the buffer
        c.encode_utf8(&mut self.buffer[self.byte_ptr..]);
        self.byte_ptr += charlen;
        self.byte_length += charlen;

        true
    }

    /// Delete character at insertion point
    ///
    /// Returns the character if it was deleted
    /// Returns `Ok(None)` if there was nothing to delete
    pub fn delete_current(&mut self) -> Result<Option<char>, LineEditError> {
        // Determine current character length
        let c = match self.current_char()? {
            Some(c) => c,
            None => return Ok(None),
        };
        let charlen = c.len_utf8();
        debug_assert!(self.byte_ptr < self.byte_length);

        // Shift everything left
        for i in self.byte_ptr..self.byte_length - charlen {
            self.buffer[i] = self.buffer[i + charlen];
        }
        self.byte_length -= charlen;

        Ok(Some(c))
    }

    /// Delete character before insertion point
    ///
    /// Returns the character if it was deleted
    /// Returns `Ok(None)` if there was nothing to delete
    pub fn delete_prev(&mut self) -> Result<Option<char>, LineEditError> {
        if self.shift_left(1)? == 0 {
            return Ok(None);
        }

        self.delete_current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn basic_insert() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("Hello world".chars());
        state.insert('!');
        assert_eq!(state.as_str()?, "Hello world!");
        Ok(())
    }

    #[test]
    fn shifting() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("Hi!".chars());
        assert_eq!(state.shift_left(1)?, 1);
        assert_eq!(state.as_str()?, "Hi!");
        state.insert_many(" there".chars());
        assert_eq!(state.as_str()?, "Hi there!");
        assert_eq!(state.shift_right(1)?, 1);
        assert_eq!(state.delete_prev()?.unwrap(), '!');
        assert_eq!(state.as_str()?, "Hi there");
        assert_eq!(state.shift_left(1)?, 1);
        assert_eq!(state.delete_prev()?.unwrap(), 'r');
        assert_eq!(state.as_str()?, "Hi thee");
        Ok(())
    }

    #[test]
    fn basic_delete() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("Hi".chars());
        assert_eq!(state.as_str()?, "Hi");
        assert_eq!(state.delete_prev()?.unwrap(), 'i');
        assert_eq!(state.as_str()?, "H");
        assert_eq!(state.delete_prev()?.unwrap(), 'H');
        assert_eq!(state.as_str()?, "");
        // Now deletion should fail
        assert!(state.delete_prev()?.is_none());
        Ok(())
    }

    #[test]
    fn check_oom_and_clear() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 4];
        let mut state = LineEditState::from_buffer(&mut buffer);
        assert_eq!(state.insert_many("Hello world".chars()), 4);
        assert!(!state.insert('!'));
        assert_eq!(state.as_str()?, "Hell");
        state.clear();
        assert_eq!(state.insert_many("123".chars()), 3);
        assert!(state.insert('4'));
        assert!(!state.insert('5'));
        assert_eq!(state.as_str()?, "1234");
        Ok(())
    }

    #[test]
    fn move_past_next_word() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("The quick    brown\tfax    ".chars());
        state.move_to_start();
        assert_eq!(state.head()?, "");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The quick");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The quick    brown");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The quick    brown\tfax");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The quick    brown\tfax    ");
        state.move_past_end_of_word()?;
        assert_eq!(state.head()?, "The quick    brown\tfax    ");
        state.move_to_prev_start_of_word()?;
        assert_eq!(state.head()?, "The quick    brown\t");
        state.move_to_prev_start_of_word()?;
        assert_eq!(state.head()?, "The quick    ");
        state.move_to_prev_start_of_word()?;
        assert_eq!(state.head()?, "The ");
        state.move_to_prev_start_of_word()?;
        assert_eq!(state.head()?, "");
        state.move_to_prev_start_of_word()?;
        assert_eq!(state.head()?, "");
        Ok(())
    }

    #[test]
    fn basic_killing() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("The quick 🦊 jamped ".chars());
        assert_eq!(state.kill_prev_word()?, "jamped ");
        assert_eq!(state.kill_prev_word()?, "🦊 ");
        assert_eq!(state.kill_prev_word()?, "quick ");
        assert_eq!(state.kill_prev_word()?, "The ");
        assert_eq!(state.kill_prev_word()?, "");
        Ok(())
    }

    #[test]
    fn kill_to_end() -> Result<(), LineEditError> {
        let mut buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(&mut buffer);
        state.insert_many("Hello World!".chars());

        assert_eq!(state.kill_to_end()?, "");
        assert_eq!(state.as_str()?, "Hello World!");

        state.move_to_prev_start_of_word()?;
        assert_eq!(state.kill_to_end()?, "World!");
        assert_eq!(state.as_str()?, "Hello ");

        state.move_to_prev_start_of_word()?;
        assert_eq!(state.kill_to_end()?, "Hello ");
        assert_eq!(state.as_str()?, "");

        assert_eq!(state.kill_to_end()?, "");
        assert_eq!(state.as_str()?, "");

        Ok(())
    }

    #[test]
    fn fuzz() -> Result<(), LineEditError> {
        let mut rng = rand::thread_rng();
        let mut buffer: Vec<u8> = vec![0; rng.gen::<usize>() % 256 + 1];
        // Shouldn't matter if it's zeroed out or not
        if rng.gen::<bool>() {
            buffer.fill_with(|| rng.gen());
        }

        let mut state = LineEditState::from_buffer(&mut buffer);

        for _ in 0..10000 {
            if rng.gen::<bool>() {
                state.insert(rng.gen());
            }
            if rng.gen::<bool>() {
                state.shift_left(rng.gen())?;
            }
            if rng.gen::<bool>() {
                let gen = rng.gen();
                state.shift_right(gen)?;
            }
            if rng.gen::<bool>() {
                state.delete_prev()?;
            }
            if rng.gen::<bool>() {
                state.delete_current()?;
            }
            if rng.gen::<bool>() {
                // Should always be valid utf-8
                state.as_str()?;
                state.tail()?;
                state.head()?;
            }
        }
        Ok(())
    }
}
