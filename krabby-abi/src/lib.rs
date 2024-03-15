//! Common types between userspace and Krabby(KabutOS kernel)
#![no_std]

/// Syscall number
#[derive(Copy, Clone, enumn::N)]
#[repr(usize)]
pub enum Syscall {
    PutChar = 1,
    PutString = 2,
    Pinfo = 3,
    Fork = 4,
    Exit = 5,
}
