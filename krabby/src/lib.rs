//! KabutOS kernel library
// We're building a kernel, so we don't have access to the standard library
#![no_std]
// Make sure everything's documented by warning when docs are missing
//#![warn(missing_docs)]
extern crate alloc;

mod allocator;
mod asm;
pub mod console;
pub mod cpu;
pub mod drivers;
pub mod errors;
pub mod filesystem;
pub mod frame;
pub mod functions;
pub mod globals;
pub mod idle;
pub mod interrupts;
pub mod mmu;
pub mod panic;
pub mod process;
pub mod scheduler;
pub mod serial;
pub mod syscalls;
pub mod timer;
pub mod trap;
pub mod userspace;
pub mod util;

#[cfg(feature = "test")]
pub mod test;

pub mod prelude {
    pub use super::{
        cpu::{HartId, InterruptId, Register},
        util::UsizeExt,
    };
    pub use super::{print, println, warn};
    pub use super::{KernelError, KernelResult};
    pub use alloc::{boxed::Box, string::String, vec::Vec};
    pub use krabby_abi::Pid;
}

pub use crate::errors::{KernelError, KernelResult};
