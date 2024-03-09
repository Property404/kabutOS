//! CLINT timer driver
//!
//! This should reliably be on any system that supports Linux because Linux requires CLINT
use crate::{
    drivers::TimerDriver,
    mmu::{map_device, PAGE_SIZE},
    util::*,
    KernelError, KernelResult,
};
use alloc::boxed::Box;
use core::{
    mem::size_of,
    ptr::{read_volatile, write_volatile},
    time::Duration,
};
use fdt::{node::FdtNode, Fdt};

// This is not a typo - it's 4094, not 4096
const CLINT_MAX_HARTS: usize = 4094;
const CLINT_MTIMECMP_REG: usize = 0x4000;
const CLINT_MTIME_REG: usize = 0xbff8;

/// CLINT timer driver
///
/// <https://chromitem-soc.readthedocs.io/en/latest/clint.html>
/// <https://osblog.stephenmarz.com/ch4.html>
#[derive(Debug)]
pub struct ClintTimerDriver {
    freq: usize,
    base_address: *mut u64,
}

impl ClintTimerDriver {
    const COMPATIBLE_STRING: &'static str = "riscv,clint0";

    /// Initialize the driver
    pub fn maybe_from_node(
        tree: &Fdt,
        node: &FdtNode,
    ) -> KernelResult<Option<Box<dyn TimerDriver>>> {
        let Some(compatible) = node.compatible() else {
            return Ok(None);
        };
        if !compatible.all().any(|v| v == Self::COMPATIBLE_STRING) {
            return Ok(None);
        }

        let reg = node
            .reg()
            .and_then(|mut v| v.next())
            .ok_or(KernelError::MissingProperty("reg"))?;
        let base_address = align_down::<PAGE_SIZE>(reg.starting_address as usize);
        let size = align_up::<PAGE_SIZE>(reg.size.unwrap_or(PAGE_SIZE));
        let base_address = map_device(base_address, size)?;

        Ok(Some(Box::new(Self {
            base_address: base_address as *mut u64,
        })))
    }

    unsafe fn write(&self, offset: usize, value: u64) {
        let address = self.base_address.wrapping_add(offset);
        unsafe { write_volatile(address, value) }
    }

    unsafe fn read(&self, offset: usize) -> u64 {
        let address = self.base_address.wrapping_add(offset);
        unsafe { read_volatile(address) }
    }
}

impl TimerDriver for ClintTimerDriver {
    fn set_alarm(&mut self, hart: usize, duration: Duration) {
        assert!(hart < CLINT_MAX_HARTS);

        let val = self.read(CLINT_MTIME_REG);
        let addr = CLINT_MTIMECMP_REG + hart * size_of::<u64>();
        self.write(
    }
}
