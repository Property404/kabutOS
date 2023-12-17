//! Kernel console
use crate::{globals, readline::Readline, serial::Serial, KernelError, KernelResult};
use core::fmt::Write;
use schmargs::Schmargs;

/// Run the kernel console
pub fn run_console() {
    let mut readline = Readline::<64>::default();
    let mut serial = Serial::new();

    loop {
        let line = readline.get_line("KabutOS➔  ").unwrap();
        if let Err(error) = parse_line(line) {
            writeln!(serial, "error: {error}").unwrap();
        }
    }
}

fn parse_line(line: &str) -> KernelResult<()> {
    let mut serial = Serial::new();
    let mut iter = line.split_whitespace();

    let Some(command) = iter.next() else {
        // No command entered
        return Ok(());
    };

    match command {
        "memdump" => {
            let Some(ptr) = iter.next() else {
                return Err(KernelError::Generic("Missing address!"));
            };
            let ptr: usize = ptr.parse()?;

            let Some(size) = iter.next() else {
                return Err(KernelError::Generic("Missing number of bytes!"));
            };
            let size: usize = size.parse()?;

            if ptr < 4096 {
                writeln!(serial, "Now you get what you deserve.")?;
                // TODO: continue;
            }

            unsafe {
                crate::functions::dump_memory(ptr as *const u8, size)?;
            };
        }

        // Display device tree
        "fdt" => {
            let args = FdtArgs::parse(iter)?;
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
        "poke" => {
            let args = PokeArgs::parse(iter)?;

            writeln!(serial, "Writing 0x{:02x} to {:p}", args.value, args.address)?;
            unsafe { core::ptr::write_volatile(args.address, args.value) };
        }

        _ => {
            return Err(KernelError::Generic("Command not found"));
        }
    }

    // (prompt: &str, buffer: &'a mut [u8]
    // parse out first word

    /*

    void run_console() {
    static char input_array[64];
    const int numbytes = readline(input_array, sizeof(input_array));
    printf("[DEBUG]%02x|%s|\n", numbytes, input_array);
    parseArray(input_array);
     */

    Ok(())
}

/// Display Device Tree
#[derive(Schmargs)]
struct FdtArgs<'a> {
    /// The path to the node to display. E.g. "/chosen"
    node: Option<&'a str>,
}

/// Write to byte in memory
#[derive(Schmargs)]
struct PokeArgs {
    /// Address to write to
    address: *mut u8,
    /// Value to write
    value: u8,
}
