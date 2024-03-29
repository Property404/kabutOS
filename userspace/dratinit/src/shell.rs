use core::fmt;
use crusty_line::CrustyLine;
use kanto::{prelude::*, sys};

struct Serial;

impl fmt::Write for Serial {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        sys::puts(string).unwrap();
        Ok(())
    }
}

impl Iterator for Serial {
    type Item = Result<char, sys::SyscallError>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(sys::getc())
    }
}

pub fn shell() {
    let mut readline = CrustyLine::<64, 8>::default();
    let prompt = "$ ";

    loop {
        let line = readline.get_line(prompt, Serial, Serial).unwrap();
        if line == "test" {
            sys::test().unwrap();
        } else {
            println!("Unknown command: {line}");
        }
    }
}
