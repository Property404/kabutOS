use crate::readline::get_line;
use crate::serial::Serial;
use core::fmt::Write;

pub fn run_console() {
    let mut serial = Serial::new();
    let mut buffer = [0u8; 64];

    loop {
        let line = get_line("KabutOS➔  ", &mut buffer).unwrap();

        let mut iter = line.split_whitespace();

        let Some(command) = iter.next() else {
            continue;
        };

        match command {
            "memdump" => {
                let Some(ptr) = iter.next() else {
                    writeln!(serial, "Missing address!");
                    continue;
                };
                let ptr: usize = ptr.parse().unwrap();

                let Some(size) = iter.next() else {
                    writeln!(serial, "Missing number of bytes!");
                    continue;
                };
                let size: usize = size.parse().unwrap();

                if size % 16 != 0 {
                    writeln!(
                        serial,
                        "Oopsie woopsie! There's a bug! Multiple of 16 ONLY!"
                    );
                    continue;
                } else if ptr < 4096 {
                    writeln!(serial, "Now you get what you deserve.");
                    // TODO: continue;
                }

                unsafe {
                    let _result = crate::functions::dump_memory(ptr as *const u8, size);
                };
            }

            _ => {
                continue;
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
    }
}
