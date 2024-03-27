//! VirtIO block driver
//!
//! <https://osblog.stephenmarz.com/ch9.html>
#![allow(dead_code)]
use crate::{
    drivers::{BlockDriver, DriverLoader, LoadContext, LoadResult},
    mmu::{self, map_device, PageAllocation, Sv39PhysicalAddress, PAGE_SIZE},
    prelude::*,
    util::*,
};
use bitflags::bitflags;
use core::{
    any::Any,
    cmp,
    mem::size_of,
    ptr::{self, read_volatile, write_volatile},
};

const EXPECTED_VERSION: u32 = 1;
const BLOCK_DEVICE_ID: u32 = 2;
const RING_SIZE: usize = 16;

const DESIRED_QUEUE_SIZE: usize = PAGE_SIZE;
const SECTOR_SIZE: usize = 512;

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
struct BlockFeatures(u32);

bitflags! {
    impl BlockFeatures: u32 {
    // Maximum size of any single segment is in size_max.
 const VIRTIO_BLK_F_SIZE_MAX =1;
    // Maximum number of segments in a request is in seg_max.
const VIRTIO_BLK_F_SEG_MAX =2;
    // Disk-style geometry specified in geometry.
const VIRTIO_BLK_F_GEOMETRY =4;
    // Device is read-only.
const VIRTIO_BLK_F_RO =5;
    // Block size of disk is in blk_size.
const VIRTIO_BLK_F_BLK_SIZE =6;
    // Cache flush command support.
const VIRTIO_BLK_F_FLUSH =9;
    // Device exports information on optimal I/O alignment.
const VIRTIO_BLK_F_TOPOLOGY =10;
    // Device can toggle its cache between writeback and writethrough modes.
const VIRTIO_BLK_F_CONFIG_WCE =11;
    // Device can support discard command, maximum discard sectors size in max_discard_sectors and maximum discard segment number in max_discard_seg.
const VIRTIO_BLK_F_DISCARD =13;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
enum BlockRequestKind {
    In = 0,
    Out = 1,
    Flush = 4,
    Discard = 11,
    WriteZeroes = 13,
}

#[derive(Debug, Default)]
#[repr(C)]
struct AvailableRing {
    flags: u16,
    index: u16,
    ring: [u16; RING_SIZE],
    used_event: u16, // Only if VIRTIO_F_EVENT_IDX
}

impl AvailableRing {
    fn push(&mut self, val: u16) {
        self.ring[self.index as usize] = val;
        self.index = (self.index + 1) % (RING_SIZE as u16);
    }
}

#[derive(Debug, Default)]
#[repr(C, align(4096))]
struct UsedRing {
    // Index of start of used descriptor chain
    id: u32,
    // Number of bytes written into device writable portion of the buffer described by the
    // descriptor chain
    len: u32,
}

#[derive(Debug)]
#[repr(C)]
struct Queue {
    descriptors: [RawDescriptor; RING_SIZE],
    available: AvailableRing,
    padding: [u8; Self::PADDING_SIZE],
    used: UsedRing,
}

impl Queue {
    const PADDING_SIZE: usize =
        PAGE_SIZE - size_of::<AvailableRing>() - size_of::<[RawDescriptor; RING_SIZE]>();
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            descriptors: Default::default(),
            available: Default::default(),
            padding: [0; Self::PADDING_SIZE],
            used: Default::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
#[repr(u8)]
enum BlockStatus {
    Ok = 0,
    IoError = 1,
    UnsupportedOperation = 2,
    #[default]
    Unwritten = 0xDA,
}

#[repr(C)]
struct BlockRequestHeader {
    kind: BlockRequestKind,
    _reserved: u32,
    sector: u64,
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
struct RawDescriptor {
    // Guest-physical address
    address: u64,
    length: u32,
    flags: DescriptorFlags,
    // "We chain unused descriptors via this"
    next: u16,
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
struct DescriptorFlags(u16);

bitflags! {
    impl DescriptorFlags: u16 {
        // This marks a buffer as continuing via the next field.
        const NEXT = 1;
        // This marks a buffer as device write-only (otherwise device read-only).
        const WRITE = 2;
        // This means the buffer contains a list of buffer descriptors
        const INDIRECT = 4;
    }
}

// From <https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html>
#[derive(Copy, Clone)]
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
    fn write<T>(self, base_address: *mut T, value: T) {
        assert!(!base_address.is_null());
        let address = base_address.wrapping_byte_add(self as usize);
        unsafe { write_volatile(address, value) }
    }
}

#[derive(Debug)]
struct VirtioBlockDriver {
    queue: Option<PageAllocation<Queue>>,
    read_only: bool,
    base_address: *mut u32,
    allocations: Vec<Box<dyn Any>>,
    index: usize,
}

unsafe impl Send for VirtioBlockDriver {}

impl VirtioBlockDriver {
    fn new(base_address: *mut u32) -> KernelResult<Self> {
        let mut device = Self {
            queue: None,
            read_only: false,
            base_address,
            allocations: Default::default(),
            index: 1,
        };

        // Reset device
        device.write(Offset::Status, Status::empty().0);

        // Set acknowledge
        device.write(Offset::Status, Status::ACKNOWLEDGE.0);

        // Set driver bit
        let status = Status(device.read(Offset::Status));
        device.write(Offset::Status, (status | Status::DRIVER).0);

        // Get host features, set guest features
        let host_features = BlockFeatures(device.read(Offset::HostFeatures));
        let read_only = host_features.contains(BlockFeatures::VIRTIO_BLK_F_RO);
        let guest_features = host_features;
        device.write(Offset::GuestFeatures, guest_features.0);

        if read_only {
            warn!("This device is read only!");
        }

        // Set features ok status bit
        let status = Status(device.read(Offset::Status));
        device.write(Offset::Status, (status | Status::FEATURES_OK).0);
        let status = Status(device.read(Offset::Status));
        if !status.contains(Status::FEATURES_OK) {
            return Err(KernelError::DriverFailure(
                "Expected guest features not supported",
            ));
        }

        // Platform-specific setup (everything before was generic except for the specific features)

        // Set queue
        let queue_num_max = usize::from_u32(device.read(Offset::QueueNumMax));
        let queue_size = cmp::min(queue_num_max, DESIRED_QUEUE_SIZE);
        device.write(Offset::QueueNum, queue_size.try_into()?);
        device.write(Offset::QueueSel, 0);
        device.write(Offset::GuestPageSize, PAGE_SIZE.try_into()?);
        let queue = mmu::zalloc(Queue::default());
        device.write(
            Offset::QueuePFN,
            usize::from(mmu::ks_vaddr_to_paddr(queue.addr())?).try_into()?,
        );

        // Set DRIVER_OK - device is now alive
        let status = Status(device.read(Offset::Status));
        device.write(Offset::Status, (status | Status::DRIVER_OK).0);

        device.queue = Some(queue);
        device.read_only = read_only;

        Ok(device)
    }

    fn push_desc(
        &mut self,
        address: Sv39PhysicalAddress,
        length: usize,
        flags: DescriptorFlags,
    ) -> KernelResult<usize> {
        self.index = (self.index + 1) % RING_SIZE;

        let desc = RawDescriptor {
            address: usize::from(address).try_into()?,
            length: length.try_into()?,
            flags,
            next: if flags.contains(DescriptorFlags::NEXT) {
                ((self.index + 1) % RING_SIZE).try_into()?
            } else {
                0
            },
        };

        self.queue.as_mut().unwrap().as_mut().descriptors[self.index] = desc;

        Ok(self.index)
    }

    fn push_sized_box<T: 'static>(
        &mut self,
        bx: Box<T>,
        flags: DescriptorFlags,
    ) -> KernelResult<usize> {
        let address: *const T = ptr::from_ref(&*bx);
        let address = mmu::ks_vaddr_to_paddr(address as usize)?;
        let length = size_of::<T>();
        // TODO: Figure out someway to dealloc
        self.allocations.push(bx);

        self.push_desc(address, length, flags)
    }

    fn push_alloc<T: ?Sized + 'static>(
        &mut self,
        bx: PageAllocation<T>,
        flags: DescriptorFlags,
    ) -> KernelResult<usize> {
        let address: *const T = bx.as_const_ptr();
        let address = mmu::ks_vaddr_to_paddr(address as *const () as usize)?;
        let length = bx.len();

        // TODO: Figure out someway to dealloc
        self.allocations.push(Box::new(bx));

        self.push_desc(address, length, flags)
    }

    fn do_operation(
        &mut self,
        buffer: *mut u8,
        buffer_len: usize,
        offset: usize,
        write: bool,
    ) -> KernelResult<()> {
        assert!(buffer_len > 0);
        if self.read_only && write {
            return Err(KernelError::DriverFailure("Writing to read-only device"));
        }

        let sector = offset / SECTOR_SIZE;

        let head_index = self.push_sized_box(
            Box::new(BlockRequestHeader {
                kind: if write {
                    BlockRequestKind::Out
                } else {
                    BlockRequestKind::In
                },
                _reserved: 0,
                sector: sector.try_into()?,
            }),
            DescriptorFlags::NEXT,
        )?;

        let buffer_paddr = mmu::ks_vaddr_to_paddr(buffer as *const () as usize)?;
        self.push_desc(
            buffer_paddr,
            buffer_len,
            DescriptorFlags::NEXT
                | if write {
                    DescriptorFlags::WRITE
                } else {
                    DescriptorFlags::empty()
                },
        )?;

        self.push_sized_box(Box::new(BlockStatus::Unwritten), DescriptorFlags::WRITE)?;

        let queue = self.queue.as_mut().unwrap().as_mut();
        queue.available.push(head_index.try_into()?);

        self.write(Offset::QueueNotify, 0);

        Ok(())
    }

    fn read(&mut self, offset: Offset) -> u32 {
        offset.read(self.base_address)
    }

    fn write(&mut self, offset: Offset, value: u32) {
        offset.write(self.base_address, value)
    }
}

impl BlockDriver for VirtioBlockDriver {}

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
