use core::arch::global_asm;

global_asm!(include_str!("asm/macros.S"));
global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/process.S"));
global_asm!(include_str!("asm/entry.S"));
