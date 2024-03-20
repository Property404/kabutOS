//! PLIC interrupt controller driver
// WIP
#![allow(dead_code)]
use crate::{
    drivers::InterruptControllerDriver,
    mmu::{map_device, PAGE_SIZE},
    util::*,
    KernelError, KernelResult,
};
use alloc::boxed::Box;
use core::{
    mem::size_of,
    ptr::{read_volatile, write_volatile},
};
use fdt::{node::FdtNode, Fdt};

/// PLIC IC driver
///
/// https://osblog.stephenmarz.com/ch5.html
#[derive(Debug)]
pub struct PlicDriver {
    base_address: *mut u32,
}

enum Offset {
    // Set priority
    Priority = 0x0000,
    // List of pending ints
    Pending = 0x1000,
    // Enable or disable int
    Enable = 0x2000,
    // Sets threshold
    Threshold = 0x20_0000,
    // Returns next interrupt or completes handling
    Claim = 0x20_0004,
}

unsafe impl Send for PlicDriver {}

impl PlicDriver {
    const COMPATIBLE_STRING: &'static str = "riscv,plic0";

    /// Initialize the driver
    pub fn maybe_from_node(
        _tree: &Fdt,
        node: &FdtNode,
    ) -> KernelResult<Option<Box<dyn InterruptControllerDriver>>> {
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

        let me = Self {
            base_address: base_address as *mut u32,
        };

        Ok(Some(Box::new(me)))
    }

    unsafe fn write(&self, offset: usize, value: u32) {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { write_volatile(address, value) }
    }

    unsafe fn read(&self, offset: usize) -> u32 {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { read_volatile(address) }
    }
}

impl InterruptControllerDriver for PlicDriver {
    fn enable(&mut self, id: usize) {
        let threshold = 0;
        let priority = 1;

        unsafe {
            let value = self.read(Offset::Enable as usize) | (1 << id);
            self.write(Offset::Enable as usize, value);
        }

        unsafe {
            self.write(Offset::Priority as usize + id * size_of::<u32>(), priority);
        }

        // Set threshold
        unsafe {
            self.write(Offset::Threshold as usize, threshold);
        }
    }
}