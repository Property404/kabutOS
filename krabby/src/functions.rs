//! Functions meant to be called from `console()`
use crate::{print, println, KernelError, KernelResult};
use core::cmp::min;
use owo_colors::{OwoColorize, Style};

fn color_byte(byte: u8) -> Style {
    let style = Style::new();
    match byte {
        0 => style.purple(),
        0x20..=0x7e => style.green(),
        0xff => style.blue(),
        _ => style,
    }
}

/// Ways a memory dump can be grouped
pub enum GroupBytesBy {
    /// No grouping
    U8,
    /// Group by 2's
    U16,
    /// Group by 4's
    U32,
    /// Group by 8's
    U64,
}

/// Show hex dump of memory.
///
/// The output is meant to look like the output of `xxd`(1)
///
/// # Safety
/// Some memory is not meant to be read. Use at your own risk.
/// Welcome to a virtual buffet of undefined behavior.
pub unsafe fn dump_memory(
    mut ptr: *const u8,
    mut size: usize,
    width: usize,
    group_by: GroupBytesBy,
) -> KernelResult<()> {
    let group = match group_by {
        GroupBytesBy::U8 => 1,
        GroupBytesBy::U16 => 2,
        GroupBytesBy::U32 => 4,
        GroupBytesBy::U64 => 8,
    };

    if width < group || width % group != 0 {
        return Err(KernelError::Generic("Width not divisible by group"));
    }

    while size > 0 {
        // Show address
        print!("{ptr:p}: ");

        // Show bytes in hex
        for minor in (0..width).step_by(group) {
            if minor < size {
                match group_by {
                    GroupBytesBy::U8 => {
                        let byte = unsafe { *(ptr.wrapping_add(minor)) };
                        let byte = byte.style(color_byte(byte));
                        print!(" {byte:02x}");
                    }
                    GroupBytesBy::U16 => {
                        print!(" {:04x}", unsafe {
                            *(ptr.wrapping_add(minor) as *const u16)
                        });
                    }
                    GroupBytesBy::U32 => {
                        print!(" {:08x}", unsafe {
                            *(ptr.wrapping_add(minor) as *const u32)
                        });
                    }
                    GroupBytesBy::U64 => {
                        print!(" {:016x}", unsafe {
                            *(ptr.wrapping_add(minor) as *const u64)
                        });
                    }
                }
            } else {
                print!(" ");
                for _ in 0..group {
                    print!("  ");
                }
            }
        }

        print!("  ");

        // Show bytes in ASCII
        for minor in 0..min(size, width) {
            let c: u8 = unsafe { *(ptr.wrapping_add(minor)) };
            let color = color_byte(c);
            let c = if (0x20..0x7f).contains(&c) {
                c as char
            } else {
                '.'
            };
            print!("{}", c.style(color));
        }

        println!();

        size = size.saturating_sub(width);
        ptr = ptr.wrapping_add(width);
    }

    Ok(())
}
