//! `#![no_std]`-friendly line editor core
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
use core::str::{self, Utf8Error};
use derive_more::{Display, From};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use utf8_parser::{Utf8ByteType, Utf8Parser, Utf8ParserError};

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

/// Error type used for this crate
#[derive(Copy, Clone, PartialEq, Eq, Debug, From, Display)]
pub enum LineEditError {
    /// Generic line editing error
    Generic(&'static str),
    /// Converted from [core::str::Utf8Error]
    #[from]
    Utf8(Utf8Error),
    /// Converted from [utf8_parser::Utf8ParserError]
    #[from]
    Utf8Parser(Utf8ParserError),
}

/// A structure used to build a line editor
///
/// This structure does not come with any way to interpret key presses or manipulate output. You must
/// do all that yourself.
///
/// This struct will not allocate memory unless provided with an implementation of [LineEditBuffer]
/// that is capable of allocating memory, e.g
/// [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)
///
/// # Example
/// ```
/// use embedded_line_edit::LineEditState;
///
/// let mut bytes = [0u8;256];
/// let mut state = LineEditState::from_buffer(bytes);
/// state.insert_many("Hello Worlf!".chars());
/// state.shift_left(1).unwrap();
/// state.delete_prev().unwrap();
/// state.insert('d');
/// state.shift_right(1).unwrap();
/// assert_eq!(state.as_str().unwrap(), "Hello World!");
/// ```
pub struct LineEditState<T> {
    buffer: T,
    byte_ptr: usize,
    byte_length: usize,
}

impl<T: LineEditBuffer> LineEditState<T> {
    /// Construct a new [LineEditState]
    ///
    /// # Panics
    /// Panics if buffer is zero sized
    pub fn from_buffer(buffer: T) -> Self {
        assert!(!buffer.as_ref().is_empty());
        Self {
            buffer,
            byte_ptr: 0,
            byte_length: 0,
        }
    }

    /// Get inner buffer as `&str`
    pub fn as_str(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(&self.buffer.as_ref()[0..self.byte_length])?)
    }

    /// Get inner buffer up to insertion point as `&str`
    pub fn head(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(&self.buffer.as_ref()[0..self.byte_ptr])?)
    }

    /// Get inner buffer from insertion point to end as `&str`
    pub fn tail(&self) -> Result<&str, LineEditError> {
        Ok(str::from_utf8(
            &self.buffer.as_ref()[self.byte_ptr..self.byte_length],
        )?)
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
            if let Some(c) = parser.push(self.buffer.as_ref()[self.byte_ptr + charlen])? {
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
        Ok(str::from_utf8(
            &self.buffer.as_ref()[self.byte_ptr..prev_length],
        )?)
    }

    /// Transpose characters
    /// This should be equivalent to ctrl+T in GNU Readline
    pub fn transpose_chars(&mut self) -> Result<(), LineEditError> {
        if self.byte_ptr == 0 {
            return Ok(());
        }

        // Cursor at end is a slight special case
        if self.byte_ptr == self.byte_length {
            self.shift_left(1)?;
        }

        self.shift_left(1)?;
        let c = self.delete_current()?;
        self.shift_right(1)?;
        if let Some(c) = c {
            self.insert(c);
        }

        Ok(())
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
            let index = self.buffer.as_ref().len() - bytes_deleted - clen;
            c.encode_utf8(&mut self.buffer.as_mut()[index..]);
            bytes_deleted += clen;
        }

        Ok(str::from_utf8(
            &self.buffer.as_ref()[self.buffer.as_ref().len() - bytes_deleted..],
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
        debug_assert!(self.buffer.as_ref().len() >= self.byte_length);
        debug_assert!(self.byte_length >= self.byte_ptr);

        let mut shifted_by = 0;
        for _ in 0..n {
            // Rewind to UTF-8 start byte
            while self.byte_ptr > 0
                && Utf8ByteType::of(self.buffer.as_ref()[self.byte_ptr - 1])?.is_continuation()
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
        debug_assert!(self.buffer.as_ref().len() >= self.byte_length);
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
                && Utf8ByteType::of(self.buffer.as_ref()[self.byte_ptr])?.is_continuation()
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

    // Request a buffer size of `needed_size` bytes or bigger
    fn request_buffer_size(&mut self, needed_size: usize) -> Result<(), ()> {
        let current_size = self.buffer.as_ref().len();
        let additional_bytes_required: isize = (needed_size as isize) - current_size as isize;
        if additional_bytes_required <= 0 {
            return Ok(());
        }
        let bytes_obtained = self
            .buffer
            .request_memory(additional_bytes_required as usize);

        if bytes_obtained < additional_bytes_required as usize {
            return Err(());
        }

        Ok(())
    }

    /// Insert a character
    ///
    /// Returns `true` if inserted, or `false` if the buffer is full
    pub fn insert(&mut self, c: char) -> bool {
        debug_assert!(self.buffer.as_ref().len() >= self.byte_length);
        debug_assert!(self.byte_length >= self.byte_ptr);

        let charlen = c.len_utf8();

        let needed_size = self.byte_length + charlen;
        if self.request_buffer_size(needed_size).is_err() {
            return false;
        }

        // First shift everything right by the character length
        for i in ((self.byte_ptr)..(self.byte_length)).rev() {
            self.buffer.as_mut()[i + charlen] = self.buffer.as_ref()[i]
        }

        // Then copy the character onto the buffer
        c.encode_utf8(&mut self.buffer.as_mut()[self.byte_ptr..]);
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
            self.buffer.as_mut()[i] = self.buffer.as_ref()[i + charlen];
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

impl<T: History> LineEditState<T> {
    /// Add new entry to the history
    pub fn new_history_entry(&mut self) {
        self.buffer.new_entry(self.byte_length);
        self.byte_length = 0;
        self.byte_ptr = 0;
    }

    /// Switch to the next history entry
    ///
    /// # Returns the size of the new buffer, or None if no new history
    pub fn next_history_entry(&mut self) -> Option<usize> {
        self.buffer.next(self.byte_length).map(|size| {
            self.byte_length = size;
            self.byte_ptr = size;
            size
        })
    }

    /// Switch to the previous history entry
    ///
    /// # Returns the size of the new buffer, or None if no old history
    pub fn prev_history_entry(&mut self) -> Option<usize> {
        self.buffer.prev(self.byte_length).map(|size| {
            self.byte_length = size;
            self.byte_ptr = size;
            size
        })
    }
}

impl<const C: usize> Default for LineEditState<[u8; C]> {
    fn default() -> Self {
        LineEditState::from_buffer([0; C])
    }
}

/// A byte array type that's allowed to be wrapped by a [LineEditState]
pub trait LineEditBuffer: AsRef<[u8]> + AsMut<[u8]> {
    /// Request `bytes` bytes of memory. The implementor is not required to grant the request.
    ///
    /// Returns the number of bytes allocated.
    fn request_memory(&mut self, _bytes: usize) -> usize {
        0
    }
}

impl<const C: usize> LineEditBuffer for [u8; C] {}

#[cfg(any(test, feature = "alloc"))]
impl LineEditBuffer for Vec<u8> {
    fn request_memory(&mut self, bytes: usize) -> usize {
        self.extend(core::iter::repeat(0).take(bytes));
        bytes
    }
}

/// Something capable of holding multiple line buffers and switching between them
pub trait History {
    /// Push a new history entry
    fn new_entry(&mut self, current_entry_size: usize);

    /// Switch to next buffer
    ///
    /// # Returns
    /// The size of the new buffer, or None if nothing left in history
    fn next(&mut self, current_entry_size: usize) -> Option<usize>;

    /// Switch to prev buffer
    ///
    /// # Returns
    /// The size of the new buffer, or None if nothing left in history
    fn prev(&mut self, current_entry_size: usize) -> Option<usize>;
}

/// A line edit buffer with a history ring
pub struct LineEditBufferWithHistoryRing<T, const HISTORY_SIZE: usize> {
    buffers: ConstGenericRingBuffer<(T, usize), HISTORY_SIZE>,
    index: usize,
}

impl<T: Default + Copy, const HISTORY_SIZE: usize> Default
    for LineEditBufferWithHistoryRing<T, HISTORY_SIZE>
{
    fn default() -> Self {
        Self::from_buffer(Default::default())
    }
}

impl<T, const HISTORY_SIZE: usize> History for LineEditBufferWithHistoryRing<T, HISTORY_SIZE>
where
    T: Clone,
{
    fn new_entry(&mut self, current_entry_size: usize) {
        self.current_mut().1 = current_entry_size;
        self.index = self.buffers.len() - 1;
        if self.current().1 > 0 {
            self.buffers.push((self.current().0.clone(), 0));
            self.index = self.buffers.len() - 1;
        }
    }

    fn next(&mut self, current_entry_size: usize) -> Option<usize> {
        self.buffers
            .get_mut(self.index)
            .expect("Bug index should be valid")
            .1 = current_entry_size;

        let index = core::cmp::min(self.index + 1, self.buffers.len().saturating_sub(1));
        if self.index == index {
            return None;
        }
        self.index = index;
        Some(
            self.buffers
                .get(index)
                .expect("Bug: Index should be valid")
                .1,
        )
    }

    fn prev(&mut self, current_entry_size: usize) -> Option<usize> {
        self.buffers
            .get_mut(self.index)
            .expect("Bug index should be valid")
            .1 = current_entry_size;

        let index = self.index.saturating_sub(1);
        if self.index == index {
            return None;
        }
        self.index = index;
        Some(
            self.buffers
                .get(index)
                .expect("Bug: Index should be valid")
                .1,
        )
    }
}

impl<T, const HISTORY_SIZE: usize> LineEditBufferWithHistoryRing<T, HISTORY_SIZE> {
    /// Construct a new [LineEditBufferWithHistoryRing] with `buffer` as the first entry
    pub fn from_buffer(buffer: T) -> Self
    where
        T: Copy,
    {
        let mut buffers = ConstGenericRingBuffer::default();
        buffers.push((buffer, 0));
        Self { buffers, index: 0 }
    }

    fn current(&self) -> &(T, usize) {
        self.buffers.get(self.index).expect("Ring buffer empty!")
    }

    fn current_mut(&mut self) -> &mut (T, usize) {
        self.buffers
            .get_mut(self.index)
            .expect("Ring buffer empty!")
    }
}

impl<T: AsRef<[u8]>, const HISTORY_SIZE: usize> AsRef<[u8]>
    for LineEditBufferWithHistoryRing<T, HISTORY_SIZE>
{
    fn as_ref(&self) -> &[u8] {
        self.current().0.as_ref()
    }
}

impl<T: AsMut<[u8]>, const HISTORY_SIZE: usize> AsMut<[u8]>
    for LineEditBufferWithHistoryRing<T, HISTORY_SIZE>
{
    fn as_mut(&mut self) -> &mut [u8] {
        self.current_mut().0.as_mut()
    }
}

impl<T: LineEditBuffer, const HISTORY_SIZE: usize> LineEditBuffer
    for LineEditBufferWithHistoryRing<T, HISTORY_SIZE>
{
    fn request_memory(&mut self, bytes: usize) -> usize {
        self.current_mut().0.request_memory(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn basic_insert() -> Result<(), LineEditError> {
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
        state.insert_many("Hello world".chars());
        state.insert('!');
        assert_eq!(state.as_str()?, "Hello world!");
        Ok(())
    }

    #[test]
    fn shifting() -> Result<(), LineEditError> {
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
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
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
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
        let buffer = [0u8; 4];
        let mut state = LineEditState::from_buffer(buffer);
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
    fn allocate_memory() -> Result<(), LineEditError> {
        let buffer = vec![0u8; 2];
        let mut state = LineEditState::from_buffer(buffer);
        state.insert_many("Hello🌈world!".chars());
        assert_eq!(state.as_str()?, "Hello🌈world!");
        assert_eq!(state.len(), 15);
        Ok(())
    }

    #[test]
    fn move_past_next_word() -> Result<(), LineEditError> {
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
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
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
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
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
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
    fn transpose_chars() -> Result<(), LineEditError> {
        let buffer = [0u8; 256];
        let mut state = LineEditState::from_buffer(buffer);
        state.insert_many("🐌Hello".chars());
        state.move_to_start();

        // Should do nothing when cursor at beginning
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "🐌Hello");

        state.shift_right(1)?;

        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "H🐌ello");
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "He🐌llo");
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "Hel🐌lo");
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "Hell🐌o");
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "Hello🐌");

        // Cursor at end - snail moves back and forth
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "Hell🐌o");
        state.transpose_chars()?;
        assert_eq!(state.as_str()?, "Hello🐌");

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

        let mut state = LineEditState::from_buffer(buffer);

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

    #[test]
    fn line_edit_with_history() -> Result<(), LineEditError> {
        let ring = LineEditBufferWithHistoryRing::<[u8; 4], 3>::default();
        let mut state = LineEditState::from_buffer(ring);

        assert_eq!(state.as_str()?, "");
        state.insert_many("One".chars());
        assert_eq!(state.as_str()?, "One");
        assert!(state.next_history_entry().is_none());
        assert!(state.prev_history_entry().is_none());

        state.new_history_entry();
        assert_eq!(state.as_str()?, "");
        state.insert_many("Two".chars());
        assert_eq!(state.as_str()?, "Two");
        assert!(state.next_history_entry().is_none());
        assert!(state.prev_history_entry().is_some());
        assert!(state.prev_history_entry().is_none());
        assert_eq!(state.as_str()?, "One");
        assert!(state.next_history_entry().is_some());
        assert_eq!(state.as_str()?, "Two");
        assert!(state.next_history_entry().is_none());
        assert_eq!(state.as_str()?, "Two");

        state.new_history_entry();
        state.insert_many("Three".chars());
        assert_eq!(state.as_str()?, "Thre");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "Two");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "One");
        assert!(state.next_history_entry().is_some());
        assert!(state.next_history_entry().is_some());

        state.new_history_entry();
        state.insert_many("Four".chars());
        assert_eq!(state.as_str()?, "Four");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "Thre");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "Two");
        // All filled up!
        assert!(state.prev_history_entry().is_none());

        state.new_history_entry();
        state.insert_many("Five".chars());
        assert_eq!(state.as_str()?, "Five");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "Four");
        assert!(state.prev_history_entry().is_some());
        assert_eq!(state.as_str()?, "Thre");
        // All filled up!
        assert!(state.prev_history_entry().is_none());

        Ok(())
    }
}
