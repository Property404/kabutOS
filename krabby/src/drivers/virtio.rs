//! VirtIO block driver
//!
//! <https://osblog.stephenmarz.com/ch9.html>
use crate::{
    drivers::{BlockDriver, DriverLoader, LoadContext, LoadResult},
    mmu::{self, map_device, Page, PAGE_SIZE},
    prelude::*,
    util::*,
};
use bitflags::bitflags;
use core::{
    fmt,
    ptr::{read_volatile, NonNull},
};

use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{device::blk::VirtIOBlk, BufferDirection, Hal, PhysAddr};

struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let virt_address: *mut [Page<PAGE_SIZE>] = mmu::zalloc_slice(pages).leak();
        let phys_address =
            usize::from(mmu::ks_vaddr_to_paddr(virt_address as *mut u8 as usize).unwrap());

        (phys_address, NonNull::new(virt_address as *mut u8).unwrap())
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        todo!("mmio_phys_to_virt")
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, _pages: usize) -> i32 {
        unsafe {
            mmu::free(vaddr.as_ptr());
        }
        // returns nonzero on error
        0
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        usize::from(mmu::ks_vaddr_to_paddr(buffer.as_ptr() as *mut u8 as usize).unwrap())
    }
    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

const EXPECTED_VERSION: u32 = 1;
const BLOCK_DEVICE_ID: u32 = 2;

const SECTOR_SIZE: usize = 512;

#[derive(Clone, Copy, Debug)]
struct Status(u32);

bitflags! {
    impl Status: u32 {
        // Indicates that the guest OS has found the device and recognized it as a valid virtio device.
        const ACKNOWLEDGE = 1;
        // Indicates that the guest OS knows how to drive the device. Note: There could be a significant (or infinite) delay before setting this bit. For example, under Linux, drivers can be loadable modules.
        const DRIVER = 2;
        // Indicates that something went wrong in the guest, and it has given up on the device. This could be an internal error, or the driver didn’t like the device for some reason, or even a fatal error during device operation
        const FAILED = 128;
        // Indicates that the driver has acknowledged all the features it understands, and feature negotiation is complete.
        const FEATURES_OK = 8;
        // Indicates that the driver is set up and ready to drive the device.
        const DRIVER_OK = 4;
        // Indicates that the device has experienced an error from which it can’t recover.
        const DEVICE_NEEDS_RESET = 64;
    }
}

// From <https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html>
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum Offset {
    MagicValue = 0x000, // R
    // 0x1 for legacy, 0x2 for modern
    Version = 0x004, // R
    // The type of device, or 0 for not loaded
    DeviceId = 0x008,         // R
    VendorId = 0x00C,         // R
    HostFeatures = 0x010,     // R
    HostFeaturesSel = 0x014,  // W
    GuestFeatures = 0x020,    // W
    GuestFeaturesSel = 0x024, // W
    GuestPageSize = 0x028,    // W
    QueueSel = 0x030,         // W
    QueueNumMax = 0x034,      // R
    QueueNum = 0x038,         // W
    QueueAlign = 0x03c,       // W
    // Guest physical page number of virtual queue
    QueuePFN = 0x040,        // RW
    QueueNotify = 0x050,     // W
    InterruptStatus = 0x060, // R
    InterruptAck = 0x064,    // W
    Status = 0x070,          // RW
    // Configuration space
    Config = 0x100, // RW
}

impl Offset {
    fn read<T>(self, base_address: *mut T) -> T {
        assert!(!base_address.is_null());
        let address = base_address.wrapping_byte_add(self as usize);
        unsafe { read_volatile(address) }
    }
}

struct VirtioBlockDriver {
    inner: VirtIOBlk<HalImpl, MmioTransport>,
}

impl fmt::Debug for VirtioBlockDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "<VirtioBlockDriver>")
    }
}

unsafe impl Send for VirtioBlockDriver {}

impl VirtioBlockDriver {
    fn new(base_address: *mut u32) -> KernelResult<Self> {
        let header = NonNull::new(base_address as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let inner = VirtIOBlk::<HalImpl, _>::new(transport).unwrap();
        Ok(Self { inner })
    }
}

impl BlockDriver for VirtioBlockDriver {
    fn acknowledge_interrupt(&mut self) -> KernelResult<()> {
        self.inner.ack_interrupt();
        Ok(())
    }

    fn start_read(&mut self, offset: usize, buffer: &mut [u8]) -> KernelResult<()> {
        assert!(aligned::<SECTOR_SIZE>(offset));
        let offset = offset / SECTOR_SIZE;
        self.inner.read_blocks(offset, buffer).unwrap();
        Ok(())
    }

    fn start_write(&mut self, offset: usize, buffer: &mut [u8]) -> KernelResult<()> {
        assert!(aligned::<SECTOR_SIZE>(offset));
        let offset = offset / SECTOR_SIZE;
        self.inner.write_blocks(offset, buffer).unwrap();
        Ok(())
    }
}

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
    let base_address = map_device(base_address, size)? as *mut u32;

    let device_id = Offset::DeviceId.read(base_address);
    let version = Offset::Version.read(base_address);

    if version != EXPECTED_VERSION {
        warn!("Found unsuitable virtio version: {version} (expected {EXPECTED_VERSION}");
        return Ok(None);
    }

    let device = match device_id {
        BLOCK_DEVICE_ID => {
            println!("Loading block driver");
            LoadResult::Block(Box::new(VirtioBlockDriver::new(base_address)?))
        }
        _ => {
            return Ok(None);
        }
    };

    Ok(Some(device))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "virtio,mmio",
    load,
};
