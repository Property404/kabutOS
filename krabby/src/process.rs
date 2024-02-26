use crate::{
    frame::TrapFrame,
    mmu::{self, Page, PageAllocation, Sv39PageTable, PAGE_SIZE},
};
use core::sync::atomic::{AtomicUsize, Ordering};

const STACK_PAGES_PER_PROCESS: usize = 2;
//const USERSPACE_VADDR_START: usize = 0x1000_0000;

/// Represents a process
#[derive(Debug)]
pub struct Process {
    pub pid: usize,
    pub code: fn(),
    pub root_page_table: PageAllocation<Sv39PageTable>,
    pub frame: PageAllocation<TrapFrame>,
    pub stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
}

impl Process {
    /// Construct a new [Process]
    pub fn new(func: fn()) -> Self {
        static PID: AtomicUsize = AtomicUsize::new(1);
        // TODO(optimization): pick a proper ordering
        // SeqCst is the safest
        let pid = PID.fetch_add(1, Ordering::SeqCst);

        let root_page_table = mmu::zalloc_page(1);

        let frame = mmu::zalloc_page(1);

        let stack = mmu::zalloc_page(STACK_PAGES_PER_PROCESS);

        Self {
            pid,
            code: func,
            root_page_table,
            frame,
            stack,
        }
    }

    /// Run the process
    pub fn run() {
        todo!()
    }
}
