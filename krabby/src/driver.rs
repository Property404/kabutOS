pub trait UartDriver {
    fn next_byte(&self) -> u8;
    fn send_byte(&self, byte: u8);

    // Todo: Query next byte if incomplete UTF-8
    fn next_char(&self) -> char {
        self.next_byte() as char
    }

    // Rust works in UTF-8 (unlike C, which generally works in ASCII), so we have to convert a Rust
    // `char` to a sequence of c `chars`(which are just bytes)
    fn send_char(&self, c: char) {
        let mut bytes = [0; 4];
        c.encode_utf8(&mut bytes);
        for byte in &bytes[0..c.len_utf8()] {
            self.send_byte(*byte)
        }
    }

    fn send_str(&self, s: &str) {
        for c in s.chars() {
            self.send_char(c);
        }
    }
}
