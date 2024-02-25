use crate::mmu::{zalloc_page, Page, PageAllocation, PAGE_SIZE};
use core::{
    cell::RefCell,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
use critical_section::Mutex;

const STACK_PAGES_PER_PROCESS: usize = 2;

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

#[derive(Debug)]
pub struct Process {
    pub pid: usize,
    pub code: fn(),
    pub frame: PageAllocation<TrapFrame>,
    pub stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
}

impl Process {
    pub fn new(func: fn()) -> Self {
        static PID: AtomicUsize = AtomicUsize::new(1);
        // TODO(optimization): pick a proper ordering
        // SeqCst is the safest
        let pid = PID.fetch_add(1, Ordering::SeqCst);
        let frame = zalloc_page(1);
        let stack = zalloc_page(STACK_PAGES_PER_PROCESS);
        Self {
            pid,
            code: func,
            frame,
            stack,
        }
    }
}
