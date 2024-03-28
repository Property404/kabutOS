//! VirtIO block driver
//!
//! <https://osblog.stephenmarz.com/ch9.html>
use crate::{
    drivers::{BlockDriver, DriverLoader, LoadContext, LoadResult},
    mmu::{self, map_device, Page, PAGE_SIZE},
    prelude::*,
    util::*,
};
use core::{fmt, ptr::NonNull};

use virtio_drivers::{
    device::blk::VirtIOBlk,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
    BufferDirection, Hal, PhysAddr,
};

const SECTOR_SIZE: usize = 512;

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
    fn new(transport: MmioTransport) -> KernelResult<Self> {
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
    let header = NonNull::new(base_address as *mut VirtIOHeader).unwrap();
    let transport = unsafe { MmioTransport::new(header) }.unwrap();

    let device = if transport.device_type() == DeviceType::Block {
        LoadResult::Block(Box::new(VirtioBlockDriver::new(transport)?))
    } else {
        return Ok(None);
    };

    Ok(Some(device))
}

pub(super) static LOADER: DriverLoader = DriverLoader {
    compatible: "virtio,mmio",
    load,
};
