use crate::{
    mmu::{self, Sv39PageTable},
    prelude::*,
};
use core::sync::atomic::{AtomicU32, Ordering};

/// Put trap frame in scratch register
pub fn set_kernel_trap_frame(hart: HartId) {
    let hart = usize::from(hart);
    // Not prepared to deal with more than 32 harts
    assert!(hart < 32);

    // Make sure we only set the trap frame once
    static HARTS_SET: AtomicU32 = AtomicU32::new(0);
    let harts_set = HARTS_SET.fetch_or(1 << hart, Ordering::SeqCst);
    if harts_set & (1 << hart) != 0 {
        panic!("Hart {hart} already set trap frame");
    }

    let mut frame = mmu::zalloc::<TrapFrame>();
    // Self referential
    frame.as_mut().kernel_frame = frame.addr();
    // Set stack and global
    frame
        .as_mut()
        .set_stack_pointer(Register::StackPointer.value());
    frame
        .as_mut()
        .set_reg(Register::GlobalPointer, Register::GlobalPointer.value());

    set_current_trap_frame(frame.leak());
}

/// Set the current trap frame
pub fn set_current_trap_frame(frame: *const TrapFrame) {
    riscv::register::sscratch::write(frame as usize);
}

/// Trap frame used per process (or by the kernel)
#[repr(C)]
#[derive(Clone, Debug)]
pub struct TrapFrame {
    /// General purpose registers
    pub regs: [usize; 32],
    /// Kernel trap frame (None for kernels)
    pub kernel_frame: usize,
    /// Process ID (0 if kernel)
    pub pid: Option<Pid>,
    /// Supervisor Address Translation/Protection register
    /// (physical address of the root page table)
    pub satp: usize,
    /// The root page table
    pub root_page_table: *mut Sv39PageTable,
}

impl TrapFrame {
    /// Get register
    pub fn get_reg(&self, reg: Register) -> usize {
        self.regs[reg as usize]
    }

    /// Set register
    pub fn set_reg(&mut self, reg: Register, value: usize) {
        self.regs[reg as usize] = value;
    }

    /// Get the stack pointer (x2 general purpose register)
    pub fn stack_pointer(&self) -> usize {
        self.get_reg(Register::StackPointer)
    }

    /// Set the stack pointer (x2 general purpose register)
    pub fn set_stack_pointer(&mut self, val: usize) {
        self.set_reg(Register::StackPointer, val)
    }

    /// Set the return value (a0)
    pub fn set_return_value<T: Into<usize> + Copy>(&mut self, val: &KernelResult<T>) {
        if let Ok(val) = val {
            self.set_reg(Register::Arg0, (*val).into());
            self.set_reg(Register::Arg1, 0);
        } else {
            self.set_reg(Register::Arg0, 0);
            self.set_reg(Register::Arg1, 1);
        }
    }

    /// Get root page table
    pub fn root_page_table(&self) -> &Sv39PageTable {
        unsafe {
            self.root_page_table
                .as_ref()
                .expect("Null dereference attempt")
        }
    }
}
