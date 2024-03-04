use core::{cell::RefCell, ptr};
use critical_section::Mutex;

/// Put trap frame in scratch register
pub fn set_kernel_trap_frame(hart: usize) {
    assert_eq!(hart, 0, "Multiple harts not supported");

    static KERNEL_TRAP_FRAME: Mutex<RefCell<TrapFrame>> = Mutex::new(RefCell::new(TrapFrame {
        regs: [0; 32],
        satp: 0,
    }));

    critical_section::with(|cs| {
        let frame: &mut TrapFrame = &mut KERNEL_TRAP_FRAME.borrow_ref_mut(cs);
        let frame = ptr::from_mut(frame);
        riscv::register::sscratch::write(frame as usize);
    });
}

/// Trap frame used per process (or by the kernel)
#[repr(C)]
#[derive(Clone, Debug)]
pub struct TrapFrame {
    /// General purpose registers
    pub regs: [usize; 32],
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
