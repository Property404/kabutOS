// Basic test
//
// Allows us to make sure page tables and whatnot get set up in CI
use crate::{frame, mmu, process::Process, scheduler, userspace, KernelResult};
use core::ptr;
use qemu_exit::QEMUExit;

// Address of qemu test device
// Not important that this is hardcoded, because we only use this on QEMU
const TEST_ADDRESS: usize = 0x100000;

pub fn test_and_exit() {
    test_kernel().unwrap();
    test_userspace().unwrap();
}

fn test_kernel() -> KernelResult<()> {
    crate::util::test();
    Ok(())
}

fn test_userspace() -> KernelResult<()> {
    let size = userspace::gary::BIN.len();
    let address = ptr::addr_of!(userspace::gary::BIN) as *const u8;
    let entry_offset = userspace::gary::ENTRY_OFFSET;

    let process = unsafe { Process::new(address, size, entry_offset)? };
    scheduler::start_with(process);
    unreachable!("Should have exited from userspace");
}

/// Quit QEMU
pub fn quit_qemu() -> KernelResult<()> {
    crate::println!("Quitting");
    frame::switch_to_kernel_frame();
    let addr = mmu::map_device(TEST_ADDRESS, 0x1000)?;
    let handle = qemu_exit::RISCV64::new(addr as u64);
    handle.exit_success();
}
