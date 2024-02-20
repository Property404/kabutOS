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

/// Show registers
pub fn show_csr_registers(names: &[&str], machine: bool) -> KernelResult<()> {
    static M_REGS: &[(&str, fn())] = &[("mstatus", || {
        let reg = riscv::register::mstatus::read();
        println!("mstatus:");
        println!("\tsie: {}", reg.sie());
        println!("\tmie: {}", reg.mie());
        println!("\tspie: {}", reg.spie());
        println!("\tmpie: {}", reg.mpie());
        println!("\tspp: {:?}", reg.spp());
        println!("\tmpp: {:?}", reg.mpp());
        println!("\tsum: {}", reg.sum());
    })];
    static S_REGS: &[(&str, fn())] = &[
        ("sstatus", || {
            let reg = riscv::register::sstatus::read();
            println!("sstatus:");
            println!("\tsie: {}", reg.sie());
            println!("\tspie: {}", reg.spie());
            println!("\tspp: {:?}", reg.spp());
            println!("\tsum: {}", reg.sum());
        }),
        ("satp", || {
            let reg = riscv::register::satp::read();
            println!("satp: {:08x}", reg.bits());
            println!("\tmode: {:?}", reg.mode());
            println!("\tasid: {:08x}", reg.asid());
            println!("\tppn: {:08x}", reg.ppn() << 12);
        }),
        ("sepc", || {
            let reg = riscv::register::sepc::read();
            println!("sepc: {reg:08x}");
        }),
        ("sie", || {
            let reg = riscv::register::sie::read();
            println!("sie: {:08x}", reg.bits());
            println!("\tssoft: {}", reg.ssoft());
            println!("\tstimer: {}", reg.stimer());
            println!("\tsext: {}", reg.sext());
        }),
        ("stvec", || {
            let reg = riscv::register::stvec::read();
            println!("stvec: {:08x}", reg.bits());
            println!("\taddress: {:08x}", reg.address());
            println!("\ttrap_mode: {:?}", reg.trap_mode());
        }),
    ];

    if names.is_empty() {
        if machine {
            for (_, func) in M_REGS {
                func()
            }
        }
        for (_, func) in S_REGS {
            func()
        }
    } else {
        'outer: for name in names {
            for (reg, func) in M_REGS.iter().chain(S_REGS.iter()) {
                if name == reg {
                    func();
                    continue 'outer;
                }
            }
            return Err(KernelError::Generic("No such register"));
        }
    }
    Ok(())
}
