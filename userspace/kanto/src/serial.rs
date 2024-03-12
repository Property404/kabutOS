//! Home of the `Serial` object - used to write to serial
use crate::sys::puts;
use core::fmt::{Error, Write};

#[doc(hidden)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Serial {}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        puts(s);
        Ok(())
    }
}

// Macros copied from <https://osblog.stephenmarz.com/ch2.html>

/// See <https://doc.rust-lang.org/std/macro.print.html>
#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
			use core::fmt::Write;
            use $crate::serial::Serial;
            let _ = write!(Serial::default(), $($args)+);
	});
}

/// See <https://doc.rust-lang.org/std/macro.println.html>
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\n")
	});
	($($args:tt)+) => ({
        use core::fmt::Write;
        use $crate::serial::Serial;
        let _ = write!(Serial::default(), $($args)+);
        let _ = write!(Serial::default(), "\n");
	});
}
