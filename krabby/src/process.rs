use crate::{
    frame::TrapFrame,
    mmu::{self, Page, PageAllocation, Sv39PageTable, PAGE_SIZE},
    KernelResult,
};
use core::sync::atomic::{AtomicUsize, Ordering};

const STACK_PAGES_PER_PROCESS: usize = 2;
const USERSPACE_VADDR_START: usize = 0x1000_0000;

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
    pub fn new(func: fn()) -> KernelResult<Self> {
        assert_eq!(
            (func as *const core::ffi::c_void) as usize & (mmu::PAGE_SIZE - 1),
            0
        );

        // TODO(optimization): pick a proper ordering
        // SeqCst is the safest
        static PID: AtomicUsize = AtomicUsize::new(1);
        let pid = PID.fetch_add(1, Ordering::SeqCst);

        let mut root_page_table = mmu::zalloc_page(1);

        // Map code
        let func_paddr = mmu::ks_vaddr_to_paddr(func as usize)?;
        mmu::map_page(
            root_page_table.as_mut(),
            USERSPACE_VADDR_START.try_into()?,
            func_paddr,
        )?;

        // Map stack
        let stack = mmu::zalloc_page(STACK_PAGES_PER_PROCESS);
        let stack_paddr = mmu::ks_vaddr_to_paddr(stack.as_const_ptr() as usize)?;
        mmu::map_range(
            root_page_table.as_mut(),
            (USERSPACE_VADDR_START + PAGE_SIZE).try_into()?,
            stack_paddr,
            STACK_PAGES_PER_PROCESS,
        )?;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc_page(1);
        frame.as_mut().satp =
            mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into();

        Ok(Self {
            pid,
            code: func,
            root_page_table,
            frame,
            stack,
        })
    }

    /// Run the process
    pub fn run(&mut self) {
        todo!()
    }
}
