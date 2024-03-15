//! KabutOS syscalls
use core::result::Result;
use krabby_abi::{KrabbyAbiError, Pid, Syscall};

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

impl From<KrabbyAbiError> for SyscallError {
    fn from(_err: KrabbyAbiError) -> Self {
        Self
    }
}

fn syscall(id: Syscall, arg0: usize, arg1: usize) -> SyscallResult<usize> {
    let res = unsafe { asm_syscall(arg0, arg1, 0, 0, 0, 0, 0, id as usize) };
    if res.err == 0 {
        Ok(res.val)
    } else {
        Err(SyscallError)
    }
}

/// Print a string (newline sold separately)
pub fn puts(s: &str) -> SyscallResult {
    syscall(
        Syscall::PutString,
        core::ptr::from_ref(s) as *const u8 as usize,
        s.len(),
    )?;
    Ok(())
}

/// Get process PID
pub fn get_pid() -> SyscallResult<Pid> {
    Ok(syscall(Syscall::Pinfo, 0, 0)?.try_into()?)
}

/// Fork process - return child PID
pub fn fork() -> SyscallResult<Option<Pid>> {
    Ok(Pid::maybe_from_usize(syscall(Syscall::Fork, 0, 0)?)?)
}

/// Exit process
pub fn exit() -> SyscallResult<()> {
    syscall(Syscall::Exit, 0, 0)?;
    Ok(())
}

/// Wait for PID
pub fn wait_pid(pid: Pid) -> SyscallResult<()> {
    syscall(Syscall::WaitPid, pid.into(), 0)?;
    Ok(())
}
