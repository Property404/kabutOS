//! Kernel console
use crate::{globals, readline::Readline, serial::Serial, KernelError, KernelResult};
use core::fmt::{Display, Write};
use schmargs::Schmargs;

/// Run the kernel console
pub fn run_console() {
    let mut readline = Readline::<64, 8>::default();
    let mut serial = Serial::new();

    loop {
        let line = readline.get_line("KabutOS➔ ").unwrap();
        if let Err(error) = parse_line(line) {
            writeln!(serial, "error: {error}").unwrap();
        }
    }
}

fn parse_line(line: &str) -> KernelResult<()> {
    let mut serial = Serial::new();
    let mut args = line.split_whitespace();

    let Some(command) = args.next() else {
        // No command entered
        return Ok(());
    };

    match command {
        HelpArgs::NAME | "?" => {
            let command_vector: [(&'static str, &'static str, &dyn Display); 4] = [
                (HelpArgs::NAME, HelpArgs::DESCRIPTION, &HelpArgs::help()),
                (
                    MemdumpArgs::NAME,
                    MemdumpArgs::DESCRIPTION,
                    &MemdumpArgs::help(),
                ),
                (FdtArgs::NAME, FdtArgs::DESCRIPTION, &FdtArgs::help()),
                (PokeArgs::NAME, PokeArgs::DESCRIPTION, &PokeArgs::help()),
            ];

            let args = HelpArgs::parse(args)?;
            if let Some(command) = args.command {
                for com in command_vector {
                    if com.0 == command {
                        writeln!(serial, "{}", com.2)?;
                        return Ok(());
                    }
                }
                writeln!(serial, "No command with name '{command}'")?;
            } else {
                for com in command_vector {
                    writeln!(serial, "{}: {}", com.0, com.1)?;
                }
            }
        }
        MemdumpArgs::NAME => {
            let args = MemdumpArgs::parse(args)?;

            if (args.start as usize) < 4096 {
                // This will crash
                writeln!(serial, "Now you get what you deserve!")?;
            }

            unsafe {
                crate::functions::dump_memory(args.start, args.len)?;
            };
        }

        // Display device tree
        FdtArgs::NAME => {
            let args = FdtArgs::parse(args)?;
            let device_tree = globals::get().device_tree;

            // If node is specified, just display that
            if let Some(node) = args.node {
                if let Some(node) = device_tree.find_node(node) {
                    writeln!(serial, "{node}")?;
                } else {
                    return Err(KernelError::Generic("Node doesn't exist"));
                }
            // Otherwise display the whole tree
            } else {
                writeln!(serial, "{device_tree:?}")?;
            }
        }

        // Write to byte address
        PokeArgs::NAME => {
            let args = PokeArgs::parse(args)?;

            writeln!(serial, "Writing 0x{:02x} to {:p}", args.value, args.address)?;
            unsafe { core::ptr::write_volatile(args.address, args.value) };
        }

        // Panic
        PanicArgs::NAME => {
            let PanicArgs { message } = PanicArgs::parse(args)?;

            panic!("{message}");
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
