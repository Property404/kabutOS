//! VirtIO block driver
//!
//! <https://osblog.stephenmarz.com/ch9.html>
#![allow(dead_code, unused_imports)]
use crate::{
    drivers::{BlockDriver, DriverLoader, LoadContext, LoadResult},
    mmu::{map_device, PAGE_SIZE},
    prelude::*,
    util::*,
};
use core::{
    mem::size_of,
    ptr::{read_volatile, write_volatile},
    time::Duration,
};

const BLOCK_DEVICE_ID: u32 = 2;

// From <https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html>
enum Offset {
    MagicValue = 0x000,
    Version = 0x004,
    DeviceId = 0x008,
    VendorId = 0x00C,
    /*
    HostFeatures = 0x010,
    HostFeaturesSel = 0x014,
    GuestFeatures = 0x020,
    GuestFeaturesSel = 0x024,
    */
}

#[derive(Debug)]
struct Driver {
    base_address: *mut u32,
}

unsafe impl Send for Driver {}

impl Driver {
    fn read(&mut self, offset: Offset) -> u32 {
        let address = self.base_address.wrapping_byte_add(offset as usize);
        unsafe { read_volatile(address) }
    }
    /*
    unsafe fn write(&mut self, offset: usize, value: u64) {
        let address = self.base_address.wrapping_byte_add(offset);
        unsafe { write_volatile(address, value) }
    }
    */
}

impl BlockDriver for Driver {}

fn load(ctx: &LoadContext) -> KernelResult<Option<LoadResult>> {
    let reg = ctx
        .node
        .reg()
        .and_then(|mut v| v.next())
        .ok_or(KernelError::MissingProperty("reg"))?;
    let base_address = align_down::<PAGE_SIZE>(reg.starting_address as usize);
    let size = align_up::<PAGE_SIZE>(reg.size.unwrap_or(PAGE_SIZE));
    // TODO: Should we have a way to unmap this?
    // We could have a RAII DeviceMapping device
    let base_address = map_device(base_address, size)?;

    let mut device = Driver {
        base_address: base_address as *mut u32,
    };

    let device_id = device.read(Offset::VendorId);
    if device_id != BLOCK_DEVICE_ID {
        return Ok(None);
    }

    println!("Loading block driver");
    Ok(Some(LoadResult::Block(Box::new(device))))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "virtio,mmio",
    load,
};
