use crate::{
    frame::{self, TrapFrame},
    mmu::{self, Page, PageAllocation, PageType, SharedAllocation, Sv39PageTable, PAGE_SIZE},
    prelude::*,
    timer::Instant,
    util::*,
};
use alloc::{sync::Arc, vec::Vec};
use core::{
    ptr,
    sync::atomic::{AtomicU16, Ordering},
};
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
    Zombie,
    /// Process is blocked on some condition
    Blocked(BlockCondition),
}

/// Condition on which a process is blocked
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockCondition {
    /// Waiting on the death of some PID
    OnDeathOfPid(Pid),
    /// Waiting for the delay to reach 0
    Until(Instant),
}

/// Represents a process
#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    /// The current top of of virtual memory. Grows as heap grows
    pub breakline: usize,
    pub pc: usize,
    pub code: Arc<SharedAllocation<[Page<PAGE_SIZE>]>>,
    pub root_page_table: PageAllocation<Sv39PageTable>,
    pub frame: PageAllocation<TrapFrame>,
    pub stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
    /// Collection of heap allocation pages
    pub heap: Vec<PageAllocation<[Page<PAGE_SIZE>]>>,
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
        let pid: Pid = {
            // TODO(optimization): pick a proper ordering
            // SeqCst is the safest
            static PID: AtomicU16 = AtomicU16::new(1);
            Pid::maybe_from_u16(PID.fetch_add(1, Ordering::SeqCst)).expect("Invalid PID generated")
        };

        let mut breakline = USERSPACE_VADDR_START.try_into()?;
        let mut root_page_table = mmu::zalloc();

        let code_paddr = mmu::ks_vaddr_to_paddr(code.addr())?;
        breakline = mmu::map_range(
            root_page_table.as_mut(),
            breakline,
            code_paddr,
            PageType::UserExecute,
            code.num_pages() * PAGE_SIZE,
        )?;

        // Map stack
        let stack = mmu::zalloc();
        let stack_paddr = mmu::ks_vaddr_to_paddr(stack.as_const_ptr() as usize)?;
        breakline = mmu::map_range(
            root_page_table.as_mut(),
            breakline,
            stack_paddr,
            PageType::UserReadWrite,
            STACK_PAGES_PER_PROCESS * PAGE_SIZE,
        )?;
        let stack_top = breakline;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc();
        frame.as_mut().pid = Some(pid);
        frame.as_mut().root_page_table = root_page_table.as_mut_ptr();
        frame.as_mut().satp =
            mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into();
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
    pub fn exit(&mut self) -> KernelResult<()> {
        self.state = ProcessState::Zombie;
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
}
