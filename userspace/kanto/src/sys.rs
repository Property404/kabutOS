//! KabutOS syscalls

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

/// Print a string (newline sold separately)
pub fn puts(s: &str) {
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

/// Get process PID
pub fn get_pid() -> usize {
    unsafe { asm_syscall(0, 0, 0, 0, 0, 0, 0, 3) }
}
