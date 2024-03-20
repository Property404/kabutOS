use core::arch::global_asm;

global_asm!(include_str!("macros.S"));
global_asm!(include_str!("trap.S"));
global_asm!(include_str!("entry.S"));
