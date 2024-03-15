use crate::{
    frame::{self, TrapFrame},
    mmu::{self, Page, PageAllocation, PageType, Sv39PageTable, PAGE_SIZE},
    prelude::*,
    util::*,
};
use core::{
    num::NonZeroU16,
    ptr,
    sync::atomic::{AtomicU16, Ordering},
};

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

/// Process ID type
#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Into, derive_more::Display)]
#[into(u16)]
pub struct Pid(NonZeroU16);

impl From<Pid> for usize {
    fn from(pid: Pid) -> Self {
        u16::from(pid).into()
    }
}

impl TryFrom<usize> for Pid {
    type Error = KernelError;
    fn try_from(pid: usize) -> KernelResult<Self> {
        let pid = pid.try_into().map_err(|_| KernelError::InvalidPid(pid))?;
        let pid = NonZeroU16::new(pid).ok_or(KernelError::InvalidPid(pid.into()))?;
        Ok(Self(pid))
    }
}

impl Pid {
    // Generate next PID
    fn next() -> Self {
        // TODO(optimization): pick a proper ordering
        // SeqCst is the safest
        static PID: AtomicU16 = AtomicU16::new(1);
        let pid = PID.fetch_add(1, Ordering::SeqCst);
        Self(pid.try_into().unwrap())
    }
}

/// Condition on which a process is blocked
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockCondition {
    /// Waiting on the death of some PID
    OnDeathOfPid(Pid),
}

/// Represents a process
#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub pc: usize,
    pub code: PageAllocation<[Page<PAGE_SIZE>]>,
    pub root_page_table: PageAllocation<Sv39PageTable>,
    pub frame: PageAllocation<TrapFrame>,
    pub stack: PageAllocation<[Page<PAGE_SIZE>; STACK_PAGES_PER_PROCESS]>,
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
        let pid = Pid::next();

        let mut root_page_table = mmu::zalloc();

        // Map code
        let mut code = mmu::zalloc_slice(align_up::<PAGE_SIZE>(code_size) / PAGE_SIZE);
        unsafe {
            ptr::copy(code_src, code.as_mut_ptr() as *mut u8, code_size);
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
        let stack_vaddr = USERSPACE_VADDR_START + code.num_pages() * PAGE_SIZE;
        mmu::map_range(
            root_page_table.as_mut(),
            stack_vaddr.try_into()?,
            stack_paddr,
            PageType::UserReadWrite,
            STACK_PAGES_PER_PROCESS * PAGE_SIZE,
        )?;

        // This doesn't need to be mapped - it's only accessed by the kernel
        let mut frame: PageAllocation<TrapFrame> = mmu::zalloc();
        frame.as_mut().pid = Some(pid);
        frame.as_mut().root_page_table = root_page_table.as_mut_ptr();
        frame.as_mut().satp =
            mmu::ks_vaddr_to_paddr(root_page_table.as_const_ptr() as usize)?.into();
        // Stack grows down, so set to top
        frame
            .as_mut()
            .set_stack_pointer(stack_vaddr + STACK_PAGES_PER_PROCESS * PAGE_SIZE);

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
            pc: USERSPACE_VADDR_START + entry_offset,
            code,
            root_page_table,
            frame,
            stack,
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

        self.state = ProcessState::Running;
    }

    /// Fork process
    pub fn fork(&self) -> KernelResult<Self> {
        let mut child = unsafe {
            Process::new(
                self.code.as_const_ptr() as *const u8,
                self.code.len(),
                0xDEADBEEF,
            )?
        };

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
