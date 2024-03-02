use crate::{
    frame::TrapFrame,
    mmu::{self, Page, PageAllocation, PageType, Sv39PageTable, PAGE_SIZE},
    println, KernelResult,
};
use core::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
use riscv::register::sstatus;

const STACK_PAGES_PER_PROCESS: usize = 2;
const USERSPACE_VADDR_START: usize = 0x1000_0000;

extern "C" {
    fn run_process(addr: usize, frame: *mut TrapFrame);
}

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
        println!("Mapping code!");
        let mut code = mmu::zalloc_slice(align_up::<PAGE_SIZE>(code_size) / PAGE_SIZE);
        unsafe {
            ptr::copy(
                code_src as *const u8,
                code.as_mut_ptr() as *mut u8,
                code_size,
            );
        }
        let code_paddr = mmu::ks_vaddr_to_paddr(code.addr())?;
        mmu::map_range(
            root_page_table.as_mut(),
            USERSPACE_VADDR_START.try_into()?,
            code_paddr,
            PageType::UserExecute,
            code.num_pages() * PAGE_SIZE,
        )?;

        // Map stack
        let stack = mmu::zalloc();
        let stack_paddr = mmu::ks_vaddr_to_paddr(stack.as_const_ptr() as usize)?;
        mmu::map_range(
            root_page_table.as_mut(),
            (USERSPACE_VADDR_START + code.num_pages() * PAGE_SIZE).try_into()?,
            stack_paddr,
            PageType::UserReadWrite,
            STACK_PAGES_PER_PROCESS * PAGE_SIZE,
        )?;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc();
        frame.as_mut().satp =
            mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into();

        // Map kernel space so we can context switch
        println!("Mapping kernel space");
        mmu::map_kernel_space(root_page_table.as_mut())?;

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
        unsafe {
            let satp = self.frame.as_ref().satp.try_into().unwrap();
            let pid = u16::try_from(self.pid).unwrap();
            println!("Setting SATP: {:08x}!", usize::from(satp));
            println!("PID: {pid:08x}!");
            mmu::set_root_page_table(pid, satp);
            sstatus::set_spp(sstatus::SPP::User);
            run_process(USERSPACE_VADDR_START, self.frame.as_mut_ptr());
        }
    }
}
