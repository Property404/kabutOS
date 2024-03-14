//! KabutOS syscalls
use core::result::Result;

#[repr(C)]
struct RawSyscallResult {
    val: usize,
    err: usize,
}

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
    ) -> RawSyscallResult;
}

/// Error type returned from kernel
#[derive(Debug)]
pub struct SyscallError;

/// Result type for syscalls
pub type SyscallResult<T = ()> = Result<T, SyscallError>;

fn syscall(id: usize, arg0: usize, arg1: usize) -> SyscallResult<usize> {
    let res = unsafe { asm_syscall(arg0, arg1, 0, 0, 0, 0, 0, id) };
    if res.err == 0 {
        Ok(res.val)
    } else {
        Err(SyscallError)
    }
}

/// Print a string (newline sold separately)
pub fn puts(s: &str) -> SyscallResult {
    syscall(2, core::ptr::from_ref(s) as *const u8 as usize, s.len())?;
    Ok(())
}

/// Get process PID
pub fn get_pid() -> SyscallResult<usize> {
    syscall(3, 0, 0)
}

/// Fork process - return child PID
pub fn fork() -> SyscallResult<usize> {
    syscall(4, 0, 0)
}
