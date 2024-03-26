//! PLIC interrupt controller driver
//!
//! <https://osblog.stephenmarz.com/ch5.html>
//! <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic-1.0.0.pdf>
// WIP
#![allow(dead_code)]
use crate::{
    drivers::{DriverLoader, InterruptControllerDriver, LoadContext, LoadResult},
    mmu::{map_device, PAGE_SIZE},
    prelude::*,
    util::*,
};
use core::{
    mem::size_of,
    ptr::{read_volatile, write_volatile},
};

#[derive(Debug)]
struct Driver {
    phandle: u32,
    base_address: *mut u32,
}

enum Offset {
    // Set priority
    Priority = 0x0000,
    // List of pending ints
    Pending = 0x1000,
    // Enable or disable int, context 1
    Enable = 0x2080,
    // Sets threshold context 1
    Threshold = 0x20_1000,
    // Returns next interrupt or completes handling for context 1
    Claim = 0x20_1004,
}

unsafe impl Send for Driver {}

impl Driver {
    unsafe fn write(&self, offset: usize, value: u32) {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { write_volatile(address, value) }
    }

    unsafe fn read(&self, offset: usize) -> u32 {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { read_volatile(address) }
    }
}

impl InterruptControllerDriver for Driver {
    fn phandle(&self) -> u32 {
        self.phandle
    }

    fn enable(&mut self, id: InterruptId) {
        let threshold = 0;
        let priority = 1;

        unsafe {
            let value = self.read(Offset::Enable as usize) | (1 << u32::from(id));
            self.write(Offset::Enable as usize, value);
        }

        unsafe {
            self.write(
                Offset::Priority as usize + usize::from(id) * size_of::<u32>(),
                priority,
            );
        }

        // Set threshold
        unsafe {
            self.write(Offset::Threshold as usize, threshold);
        }
    }

    fn next(&mut self) -> Option<InterruptId> {
        let claim = unsafe { self.read(Offset::Claim as usize) };

        if claim == 0 {
            return None;
        }

        unsafe { self.write(Offset::Claim as usize, claim) };

        Some(claim.try_into().unwrap())
    }
}

fn load(info: &LoadContext) -> KernelResult<Option<LoadResult>> {
    let phandle: u32 = info
        .node
        .property("phandle")
        .ok_or(KernelError::MissingProperty("phandle"))?
        .as_usize()
        .ok_or(KernelError::Generic("Invalid phandle"))?
        .try_into()
        .expect("Expect usize to hold u32");

    let reg = info
        .node
        .reg()
        .and_then(|mut v| v.next())
        .ok_or(KernelError::MissingProperty("reg"))?;
    let base_address = align_down::<PAGE_SIZE>(reg.starting_address as usize);
    let size = align_up::<PAGE_SIZE>(reg.size.unwrap_or(PAGE_SIZE));
    let base_address = map_device(base_address, size)?;

    let device = Driver {
        phandle,
        base_address: base_address as *mut u32,
    };

    Ok(Some(LoadResult::InterruptController(Box::new(device))))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "riscv,plic0",
    load,
};
