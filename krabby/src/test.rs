// Basic test
//
// Allows us to make sure page tables and whatnot get set up in CI
use crate::{mmu, KernelResult};
use qemu_exit::QEMUExit;

// Address of qemu test device
// Not important that this is hardcoded, because we only use this on QEMU
const TEST_ADDRESS: usize = 0x100000;

pub fn test_and_exit() {
    test_inner().unwrap();
}

fn test_inner() -> KernelResult<()> {
    crate::util::test();

    quit_qemu()
}

fn quit_qemu() -> KernelResult<()> {
    let addr = mmu::map_device(TEST_ADDRESS, 0x1000)?;
    let handle = qemu_exit::RISCV64::new(addr as u64);
    handle.exit_success();
}
