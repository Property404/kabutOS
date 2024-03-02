use crate::{
    frame::TrapFrame,
    mmu::{self, Page, PageAllocation, Sv39PageTable, PAGE_SIZE},
    KernelResult,
};
use core::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

const STACK_PAGES_PER_PROCESS: usize = 2;
const USERSPACE_VADDR_START: usize = 0x1000_0000;

const fn align_up<const SIZE: usize>(val: usize) -> usize {
    let mut rv = align_down::<SIZE>(val);
    if val % SIZE != 0 {
        rv += SIZE;
    }
    rv
}

const fn align_down<const SIZE: usize>(val: usize) -> usize {
    assert!(SIZE.is_power_of_two());
    SIZE * (val / SIZE)
}

/// Represents a process
#[derive(Debug)]
pub struct Process {
    pub pid: usize,
    pub code: PageAllocation<[Page<PAGE_SIZE>]>,
    pub root_page_table: PageAllocation<Sv39PageTable>,
    pub frame: PageAllocation<TrapFrame>,
    pub stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
}

impl Process {
    /// Construct a new [Process]
    pub fn new(code_src: *const (), code_size: usize) -> KernelResult<Self> {
        // TODO(optimization): pick a proper ordering
        // SeqCst is the safest
        static PID: AtomicUsize = AtomicUsize::new(1);
        let pid = PID.fetch_add(1, Ordering::SeqCst);

        let mut root_page_table = mmu::zalloc();

        // Map code
        let mut code = mmu::zalloc_slice(align_up::<PAGE_SIZE>(code_size) / PAGE_SIZE);
        unsafe {
            ptr::copy(
                code_src as *const u8,
                code.as_mut_ptr() as *mut u8,
                code.num_pages() * PAGE_SIZE,
            );
        }
        let code_paddr = mmu::ks_vaddr_to_paddr(code.addr())?;
        mmu::map_page(
            root_page_table.as_mut(),
            USERSPACE_VADDR_START.try_into()?,
            code_paddr,
        )?;

        // Map stack
        // TODO: make slice
        let stack = mmu::zalloc();
        let stack_paddr = mmu::ks_vaddr_to_paddr(stack.as_const_ptr() as usize)?;
        mmu::map_range(
            root_page_table.as_mut(),
            (USERSPACE_VADDR_START + PAGE_SIZE).try_into()?,
            stack_paddr,
            STACK_PAGES_PER_PROCESS,
        )?;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc();
        frame.as_mut().satp =
            mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into();

        Ok(Self {
            pid,
            code,
            root_page_table,
            frame,
            stack,
        })
    }

    /// Run the process
    pub fn run(&mut self) {
        todo!("Set PC to code")
    }
}
