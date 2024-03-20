use crate::{
    frame::{self, TrapFrame},
    mmu::{self, Page, PageAllocation, PageType, SharedAllocation, Sv39PageTable, PAGE_SIZE},
    prelude::*,
    timer::Instant,
    util::*,
};
use alloc::{sync::Arc, vec::Vec};
use core::ptr;
use krabby_abi::ProcessResult;
use riscv::register::sstatus;

const STACK_PAGES_PER_PROCESS: usize = 2;
const USERSPACE_VADDR_START: usize = 0xf000_0000;

/// Process state
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is waiting to be run
    Ready,
    /// Process is running
    Running,
    /// Process has been terminated but not yet reaped
    Zombie(ProcessResult),
    /// Process is blocked on some condition
    Blocked(BlockCondition),
}

/// Condition on which a process is blocked
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockCondition {
    /// Waiting on the death of some PID
    OnDeathOfPid(Pid),
    /// Waiting on uart character available
    OnUart(InterruptId),
    /// Waiting for the delay to reach 0
    Until(Instant),
}

/// Represents a process
#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub pc: usize,
    pub frame: PageAllocation<TrapFrame>,
    // The current top of of virtual memory. Grows as heap grows
    breakline: usize,
    code: Arc<SharedAllocation<[Page<PAGE_SIZE>]>>,
    root_page_table: PageAllocation<Sv39PageTable>,
    stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
    /// Collection of heap allocation pages
    heap: Vec<PageAllocation<[Page<PAGE_SIZE>]>>,
}

impl Process {
    /// Construct a new [Process]
    ///
    /// # Safety
    /// `code_src` and `code_size` must be valid
    pub unsafe fn new(
        code_src: *const u8,
        code_size: usize,
        entry_offset: usize,
    ) -> KernelResult<Self> {
        // Map code
        let mut code = mmu::zalloc_slice(align_up::<PAGE_SIZE>(code_size) / PAGE_SIZE);
        unsafe {
            ptr::copy(code_src, code.as_mut_ptr() as *mut u8, code_size);
        }
        let code = code.into_shared();
        let pc = USERSPACE_VADDR_START + entry_offset;
        Self::with_code_and_pc(code, pc)
    }

    fn with_code_and_pc(
        code: Arc<SharedAllocation<[Page<PAGE_SIZE>]>>,
        pc: usize,
    ) -> KernelResult<Self> {
        let pid = Pid::generate();

        let mut breakline = USERSPACE_VADDR_START.try_into()?;
        let mut root_page_table = mmu::zalloc(Sv39PageTable::new());

        let code_paddr = mmu::ks_vaddr_to_paddr(code.addr())?;
        breakline = mmu::map_range(
            root_page_table.as_mut(),
            breakline,
            code_paddr,
            PageType::UserExecute,
            code.num_pages() * PAGE_SIZE,
        )?;

        // Skip a page for the stack guard
        breakline = breakline.offset(PAGE_SIZE as isize)?;

        // Map stack
        let stack = mmu::zalloc(Default::default());
        let stack_paddr = mmu::ks_vaddr_to_paddr(stack.as_const_ptr() as usize)?;
        breakline = mmu::map_range(
            root_page_table.as_mut(),
            breakline,
            stack_paddr,
            PageType::UserReadWrite,
            STACK_PAGES_PER_PROCESS * PAGE_SIZE,
        )?;
        let stack_top = breakline;

        // Skip a page for the heap guard
        breakline = breakline.offset(PAGE_SIZE as isize)?;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc(TrapFrame {
            regs: Default::default(),
            pid: Some(pid),
            root_page_table: root_page_table.as_mut_ptr(),
            satp: mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into(),
            kernel_frame: ptr::null(),
        });
        // Stack grows down, so set to top
        frame.as_mut().set_stack_pointer(stack_top.into());

        // Map kernel space so we can context switch
        mmu::map_kernel_space(root_page_table.as_mut())?;

        // TODO: Remove this hardcoded address
        mmu::map_range(
            root_page_table.as_mut(),
            (0x1000_0000_usize).try_into().unwrap(),
            (0x1000_0000_usize).try_into().unwrap(),
            PageType::Kernel,
            PAGE_SIZE,
        )?;

        Ok(Self {
            pid,
            state: ProcessState::Ready,
            breakline: usize::from(breakline),
            pc,
            code,
            root_page_table,
            frame,
            stack,
            heap: Vec::new(),
        })
    }

    /// Switch process to ready
    pub fn pause(&mut self) {
        self.state = ProcessState::Ready;
    }

    /// Switch process to running
    pub fn switch(&mut self) {
        let kernel_trap_frame = riscv::register::sscratch::read() as *const TrapFrame;
        assert!(!kernel_trap_frame.is_null());
        self.frame.as_mut().kernel_frame = unsafe { (*kernel_trap_frame).kernel_frame };

        // Set page tables
        let satp = self.frame.as_ref().satp.try_into().unwrap();
        let pid = u16::from(self.pid);
        mmu::set_root_page_table(pid, satp);

        frame::set_current_trap_frame(self.frame.as_mut_ptr());

        unsafe {
            sstatus::set_spp(sstatus::SPP::User);
        }

        self.state = ProcessState::Running;
    }

    /// Fork process
    pub fn fork(&self) -> KernelResult<Self> {
        let mut child = Process::with_code_and_pc(self.code.clone(), 0xDEADBEEF)?;

        // Copy over registers
        child.pc = self.pc;
        for (i, reg) in self.frame.as_ref().regs.iter().enumerate() {
            child.frame.as_mut().regs[i] = *reg;
        }

        // Copy stack
        for (pindex, page) in self.stack.as_ref().iter().enumerate() {
            for (bindex, byte) in page.0.iter().enumerate() {
                child.stack.as_mut()[pindex].0[bindex] = *byte;
            }
        }

        Ok(child)
    }

    /// Terminate the process
    pub fn exit(&mut self, res: ProcessResult) -> KernelResult<()> {
        self.state = ProcessState::Zombie(res);
        Ok(())
    }

    /// Return true if blocked
    pub fn is_blocked(&mut self) -> bool {
        matches!(self.state, ProcessState::Blocked(_))
    }

    /// Block process on some condition
    pub fn block(&mut self, condition: BlockCondition) {
        self.pause();
        self.state = ProcessState::Blocked(condition);
    }

    /// Unblock process
    pub fn unblock(&mut self) {
        assert!(self.is_blocked());
        self.state = ProcessState::Ready;
    }

    /// Get the heap breakline
    pub fn breakline(&self) -> usize {
        self.breakline
    }

    /// Allocate user pages
    ///
    /// Returns the new breakline
    pub fn request_memory(&mut self, bytes: usize) -> KernelResult<usize> {
        if bytes == 0 {
            return Ok(self.breakline);
        }

        let num_pages = align_up::<PAGE_SIZE>(bytes) / PAGE_SIZE;
        let new_allocation = mmu::zalloc_slice(num_pages);
        let paddr = mmu::ks_vaddr_to_paddr(new_allocation.addr())?;

        self.breakline = mmu::map_range(
            self.root_page_table.as_mut(),
            self.breakline.try_into()?,
            paddr,
            PageType::UserReadWrite,
            new_allocation.num_pages() * PAGE_SIZE,
        )?
        .into();

        self.heap.push(new_allocation);

        Ok(self.breakline)
    }
}
