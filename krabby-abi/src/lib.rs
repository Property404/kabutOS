//! Common types between userspace and Krabby(KabutOS kernel)
#![no_std]
mod error;
mod pid;

pub use error::{KrabbyAbiError, ProcessError, ProcessResult};
pub use pid::Pid;

/// Syscall number
#[derive(Copy, Clone, enumn::N)]
#[repr(usize)]
pub enum Syscall {
    PutChar = 1,
    PutString,
    Pinfo,
    Fork,
    Exit,
    WaitPid,
    Sleep,
    RequestMemory,
}
