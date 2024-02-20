//! Kernel console
use crate::{
    functions::GroupBytesBy, globals, println, readline::Readline, KernelError, KernelResult,
};
use core::fmt::Display;
use schmargs::Schmargs;

/// Run the kernel console
pub fn run_console() {
    let mut readline = Readline::<64, 8>::default();

    loop {
        let line = readline.get_line("KabutOS➔ ").unwrap();
        if let Err(error) = parse_line(line) {
            println!("error: {error}");
        }
    }
}

fn parse_line(line: &str) -> KernelResult<()> {
    let mut args = line.split_whitespace();

    let Some(command) = args.next() else {
        // No command entered
        return Ok(());
    };

    match command {
        HelpArgs::NAME | "?" => {
            let command_vector: [(&'static str, &'static str, &dyn Display); 6] = [
                (HelpArgs::NAME, HelpArgs::DESCRIPTION, &HelpArgs::help()),
                (
                    MemdumpArgs::NAME,
                    MemdumpArgs::DESCRIPTION,
                    &MemdumpArgs::help(),
                ),
                (FdtArgs::NAME, FdtArgs::DESCRIPTION, &FdtArgs::help()),
                (PokeArgs::NAME, PokeArgs::DESCRIPTION, &PokeArgs::help()),
                (PanicArgs::NAME, PanicArgs::DESCRIPTION, &PanicArgs::help()),
                (CsrArgs::NAME, CsrArgs::DESCRIPTION, &CsrArgs::help()),
            ];

            let args = HelpArgs::parse(args)?;
            if let Some(command) = args.command {
                for com in command_vector {
                    if com.0 == command {
                        println!("{}", com.2);
                        return Ok(());
                    }
                }
                println!("No command with name '{command}'");
            } else {
                for com in command_vector {
                    println!("{}: {}", com.0, com.1);
                }
            }
        }
        MemdumpArgs::NAME => {
            let args = MemdumpArgs::parse(args)?;

            let group_by = match args.group_by {
                8 => GroupBytesBy::U8,
                16 => GroupBytesBy::U16,
                32 => GroupBytesBy::U32,
                64 => GroupBytesBy::U64,
                _ => {
                    return Err(KernelError::Generic(
                        "--group-by should be 8, 16, 32, or 64",
                    ));
                }
            };

            if (args.start as usize) < 4096 {
                // This will crash
                println!("Now you get what you deserve!");
            }

            unsafe {
                crate::functions::dump_memory(args.start, args.len, args.width, group_by)?;
            };
        }

        // Display device tree
        FdtArgs::NAME => {
            let args = FdtArgs::parse(args)?;
            let device_tree = globals::get().device_tree;

            // If node is specified, just display that
            if let Some(node) = args.node {
                if let Some(node) = device_tree.find_node(node) {
                    println!("{node}");
                } else {
                    return Err(KernelError::Generic("Node doesn't exist"));
                }
            // Otherwise display the whole tree
            } else {
                println!("{device_tree:?}");
            }
        }

        // Write to byte address
        PokeArgs::NAME => {
            let args = PokeArgs::parse(args)?;

            println!("Writing 0x{:02x} to {:p}", args.value, args.address);
            unsafe { core::ptr::write_volatile(args.address, args.value) };
        }

        // Panic
        PanicArgs::NAME => {
            let PanicArgs { message } = PanicArgs::parse(args)?;

            panic!("{message}");
        }

        // Control and Status registers
        CsrArgs::NAME => {
            let CsrArgs {} = CsrArgs::parse(args)?;

            let sstatus = riscv::register::sstatus::read();
            println!("sstatus:");
            println!("\tsie: {}", sstatus.sie());
            println!("\tspie: {}", sstatus.spie());
            println!("\tspp: {:?}", sstatus.spp());
            println!("\tsum: {}", sstatus.sum());
            let satp = riscv::register::satp::read();
            println!("satp: {:08x}", satp.bits());
            println!("\tmode: {:?}", satp.mode());
            println!("\tasid: {:08x}", satp.asid());
            println!("\tppn: {:08x}", satp.ppn() << 12);
            let sepc = riscv::register::sepc::read();
            println!("sepc: {sepc:08x}");
            let sie = riscv::register::sie::read();
            println!("sie: {:08x}", sie.bits());
            println!("\tssoft: {}", sie.ssoft());
            println!("\tstimer: {}", sie.stimer());
            println!("\tsext: {}", sie.sext());
            let stvec = riscv::register::stvec::read();
            println!("stvec: {:08x}", stvec.bits());
            println!("\taddress: {:08x}", stvec.address());
            println!("\ttrap_mode: {:?}", stvec.trap_mode());
        }

        _ => {
            return Err(KernelError::Generic("Command not found"));
        }
    }

    Ok(())
}

/// Display help
#[derive(Schmargs)]
#[schmargs(name = "help")]
struct HelpArgs<'a> {
    /// Command for which to show help
    command: Option<&'a str>,
}

/// Dump memory at address
#[derive(Schmargs)]
#[schmargs(name = "memdump")]
struct MemdumpArgs {
    /// Width of each row in bytes
    #[arg(short, long, default_value = 16)]
    width: usize,
    /// Group bytes by power of two
    #[arg(short, long, default_value = 8)]
    group_by: usize,
    /// Starting memory address
    start: *const u8,
    /// Number of bytes to read
    #[arg(default_value = 0x100)]
    len: usize,
}

/// Force a kernel panic
#[derive(Schmargs)]
#[schmargs(name = "panic")]
struct PanicArgs<'a> {
    /// Messsage to panic with
    message: &'a str,
}

/// Display Device Tree
#[derive(Schmargs)]
#[schmargs(name = "fdt")]
struct FdtArgs<'a> {
    /// The path to the node to display. E.g. "/chosen"
    node: Option<&'a str>,
}

/// Write to byte in memory
#[derive(Schmargs)]
#[schmargs(name = "poke")]
struct PokeArgs {
    /// Address to write to
    address: *mut u8,
    /// Value to write
    value: u8,
}

/// Show control and status regs
#[derive(Schmargs)]
#[schmargs(name = "csr")]
struct CsrArgs {}
