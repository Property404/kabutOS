//! Common types between userspace and Krabby(KabutOS kernel)
#![no_std]
mod error;
mod pid;
mod sys;

pub mod fs;

pub use error::{KrabbyAbiError, ProcessError, ProcessResult};
pub use pid::Pid;
pub use sys::Syscall;
