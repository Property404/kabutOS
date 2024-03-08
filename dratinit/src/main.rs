#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("crt.S"));

extern "C" {
    fn asm_syscall(
        a0: usize,
        a1: usize,
        a2: usize,
        a3: usize,
        a4: usize,
        a5: usize,
        a6: usize,
        a7: usize,
    );
}

fn putchar(c: char) {
    unsafe {
        asm_syscall(c as usize, 0, 0, 0, 0, 0, 0, 1);
    }
}

fn puts(s: &str) {
    for c in s.chars() {
        putchar(c);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() {
    puts("Hello, world!\n");
    for _ in 0..5 {
        putchar('!');
    }
    putchar('\n');
}
