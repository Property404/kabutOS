//! Functions meant to be called from `console()`
use crate::{serial::Serial, KernelResult};
use core::fmt::Write;
use owo_colors::{OwoColorize, Style};

fn color_byte(byte: u8) -> Style {
    let style = Style::new();
    match byte {
        0 => style.red(),
        0x20..=0x7e => style.green(),
        0xff => style.blue(),
        _ => style,
    }
}

/// Show hex dump of memory.
///
/// The output is meant to look like the output of `xxd`(1)
///
/// # Safety
/// Some memory is not meant to be read. Use at your own risk.
/// Welcome to a virtual buffet of undefined behavior.
pub unsafe fn dump_memory(mut ptr: *const u8, mut size: usize) -> KernelResult<()> {
    const WIDTH: usize = 16;
    let mut serial = Serial::new();

    while size > 0 {
        // Show address
        write!(serial, "{ptr:p}: ")?;

        // Show bytes in hex
        for minor in (0..WIDTH).step_by(2) {
            let (byte1, byte2) =
                unsafe { (*(ptr.wrapping_add(minor)), *(ptr.wrapping_add(minor + 1))) };
            let byte1 = byte1.style(color_byte(byte1));
            let byte2 = byte2.style(color_byte(byte2));
            write!(serial, " {byte1:02x}{byte2:02x}")?
        }

        write!(serial, "  ")?;

        // Show bytes in ASCII
        for minor in 0..WIDTH {
            let c: u8 = unsafe { *(ptr.wrapping_add(minor)) };
            let color = color_byte(c);
            let c = if (0x20..0x7f).contains(&c) {
                c as char
            } else {
                '.'
            };
            write!(serial, "{}", c.style(color))?;
        }

        writeln!(serial)?;

        size -= WIDTH;
        ptr = ptr.wrapping_add(WIDTH);
    }

    Ok(())
}
