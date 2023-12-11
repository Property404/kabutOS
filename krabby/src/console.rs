//! Kernel console
use crate::{globals, readline::get_line, serial::Serial, KernelError, KernelResult};
use core::fmt::Write;

/// Run the kernel console
pub fn run_console() {
    let mut buffer = [0u8; 64];
    let mut serial = Serial::new();

    loop {
        let line = get_line("KabutOS➔  ", &mut buffer).unwrap();
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
        "dt" => {
            let device_tree = globals::get().device_tree;
            writeln!(serial, "{device_tree:?}")?;
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
