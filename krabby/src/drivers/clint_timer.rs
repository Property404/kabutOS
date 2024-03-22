//! CLINT timer driver
//!
//! <https://chromitem-soc.readthedocs.io/en/latest/clint.html>
//! <https://osblog.stephenmarz.com/ch4.html>
//!
//! This should reliably be on any system that supports Linux because Linux requires CLINT
use crate::{
    drivers::{DriverLoader, LoadContext, LoadResult, TimerDriver},
    mmu::{map_device, PAGE_SIZE},
    prelude::*,
    util::*,
};
use alloc::boxed::Box;
use core::{mem::size_of, ptr::write_volatile, time::Duration};

const NANOS_PER_SECOND: u128 = 1_000_000_000;
// This is not a typo - it's 4094, not 4096
const CLINT_MAX_HARTS: usize = 4094;
const CLINT_MTIMECMP_REG: usize = 0x4000;
const CLINT_MTIME_REG: usize = 0xbff8;

#[derive(Debug)]
struct Driver {
    freq: usize,
    // mtime and mtimecmp havew 64-bit precision regardless of architecture
    base_address: *mut u64,
}

unsafe impl Send for Driver {}

impl Driver {
    unsafe fn write(&self, offset: usize, value: u64) {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { write_volatile(address, value) }
    }
}

impl TimerDriver for Driver {
    fn set_alarm(&mut self, hart: HartId, duration: Duration) {
        let hart = usize::from(hart);
        assert!(hart < CLINT_MAX_HARTS);

        let cycles = duration.as_nanos() * (self.freq as u128) / NANOS_PER_SECOND;
        let val = cycles as u64;
        let offset = CLINT_MTIMECMP_REG + hart * size_of::<u64>();

        unsafe {
            self.write(CLINT_MTIME_REG, 0);
            self.write(offset, val);
        }
    }
}

fn load(info: &LoadContext) -> KernelResult<LoadResult> {
    let reg = info
        .node
        .reg()
        .and_then(|mut v| v.next())
        .ok_or(KernelError::MissingProperty("reg"))?;
    let base_address = align_down::<PAGE_SIZE>(reg.starting_address as usize);
    let size = align_up::<PAGE_SIZE>(reg.size.unwrap_or(PAGE_SIZE));
    let base_address = map_device(base_address, size)?;

    let freq = info
        .fdt
        .find_node("/cpus")
        .ok_or(KernelError::MissingProperty("cpus"))?
        .property("timebase-frequency")
        .ok_or(KernelError::MissingProperty("timebase-frequency"))?
        .as_usize()
        .ok_or(KernelError::Generic("Invalid timebase frequency"))?;

    let device = Driver {
        freq,
        base_address: base_address as *mut u64,
    };

    Ok(LoadResult::Timer(Box::new(device)))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "riscv,clint0",
    load,
};
