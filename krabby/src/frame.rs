use crate::mmu;
use core::sync::atomic::{AtomicU32, Ordering};

/// Put trap frame in scratch register
pub fn set_kernel_trap_frame(hart: usize) {
    // Not prepared to deal with more than 32 harts
    assert!(hart < 32);

    // Make sure we only set the trap frame once
    static HARTS_SET: AtomicU32 = AtomicU32::new(0);
    let harts_set = HARTS_SET.fetch_or(1 << hart, Ordering::SeqCst);
    if harts_set & (1 << hart) != 0 {
        panic!("Hart {hart} already set trap frame");
    }

    let mut frame = mmu::zalloc::<TrapFrame>();
    frame.as_mut().kernel_frame = 0xDEADBEEF;
    riscv::register::sscratch::write(frame.leak() as usize);
}

/// Trap frame used per process (or by the kernel)
#[repr(C)]
#[derive(Clone, Debug)]
pub struct TrapFrame {
    /// General purpose registers
    pub regs: [usize; 32],
    /// Kernel trap frame (None for kernels)
    pub kernel_frame: usize,
    /// Supervisor Address Translation/Protection register
    /// (Where is the root page table for this process)
    pub satp: usize,
}

impl TrapFrame {
    /// Set the stack pointer (x2 general purpose register)
    pub fn set_stack_pointer(&mut self, val: usize) {
        self.regs[2] = val
    }

    /// Set the global pointer (x3 general purpose register)
    pub fn set_global_pointer(&mut self, val: usize) {
        self.regs[3] = val
    }
}
