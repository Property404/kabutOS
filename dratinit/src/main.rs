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
    ) -> usize;
}

fn puts(s: &str) {
    unsafe {
        asm_syscall(
            core::ptr::from_ref(s) as *const u8 as usize,
            s.len(),
            0,
            0,
            0,
            0,
            0,
            2,
        );
    }
}

fn get_pid() -> usize {
    unsafe { asm_syscall(0, 0, 0, 0, 0, 0, 0, 3) }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    puts("Userspace panicking!\n");
    loop {}
}

#[no_mangle]
extern "C" fn main() {
    let pid = get_pid();
    puts("Hello, Sweetie!\n");
    for _ in 0..4 {
        if pid % 2 == 0 {
            puts("Howdy\n");
        } else {
            puts("Hmm!\n");
        }
    }
}
