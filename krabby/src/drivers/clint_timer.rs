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

const NANOS_PER_SECOND: u128 = 1_000_000_000;
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
    // mtime and mtimecmp havew 64-bit precision regardless of architecture
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

        let freq = tree
            .find_node("/cpus")
            .ok_or(KernelError::MissingProperty("cpus"))?
            .property("timebase-frequency")
            .ok_or(KernelError::MissingProperty("timebase-frequency"))?
            .as_usize()
            .ok_or(KernelError::Generic("Invalid tiembase frequency"))?;

        Ok(Some(Box::new(Self {
            freq,
            base_address: base_address as *mut u64,
        })))
    }

    unsafe fn write(&self, offset: usize, value: u64) {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { write_volatile(address, value) }
    }

    unsafe fn read(&self, offset: usize) -> u64 {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { read_volatile(address) }
    }
}

impl TimerDriver for ClintTimerDriver {
    fn set_alarm(&mut self, hart: usize, duration: Duration) {
        assert!(hart < CLINT_MAX_HARTS);

        let cycles = duration.as_nanos() * (self.freq as u128) / NANOS_PER_SECOND;
        let time = unsafe { self.read(CLINT_MTIME_REG) };
        let val = time + cycles as u64;
        let offset = CLINT_MTIMECMP_REG + hart * size_of::<u64>();

        unsafe {
            self.write(offset, val);
        }
    }
}
