//! MMU and paging setup
//!
//! Most of this is based off <https://osblog.stephenmarz.com/ch3.2.html>
use crate::{println, serial::Serial, KernelError, KernelResult};
use bilge::prelude::*;
use core::{
    cell::{Cell, RefCell},
    ffi::c_void,
    fmt::{Debug, Write},
    ptr,
};
use critical_section::Mutex;
use page_alloc::RecordsPage;

/// The root kernel-space page table
#[link_section = ".kernel_root_page_table"]
pub static ROOT_PAGE_TABLE: Mutex<RefCell<Sv39PageTable>> =
    Mutex::new(RefCell::new(Sv39PageTable::new()));

// Offset between kernel space and physical memory
pub static PHYSICAL_MEMORY_OFFSET: Mutex<Cell<isize>> = Mutex::new(Cell::new(0));

extern "C" {
    static mut table_heap_bottom: RecordsPage<PAGE_SIZE>;
    static table_heap_top: c_void;
    static kernel_start: c_void;
    static stack_bottom: c_void;
    static stack_top: c_void;
}

pub const PAGE_SIZE: usize = 4096;
const MAX_VIRTUAL_ADDRESS: usize = (1 << 39) - 1;
const MAX_PHYSICAL_ADDRESS: usize = (1 << 56) - 1;
const ENTRIES_IN_PAGE_TABLE: usize = 512;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PageType {
    UserReadOnly,
    UserReadWrite,
    UserExecute,
    Kernel,
}

impl PageType {
    const fn read(self) -> bool {
        true
    }

    const fn write(self) -> bool {
        matches!(self, Self::UserReadWrite | Self::Kernel)
    }

    const fn execute(self) -> bool {
        matches!(self, Self::UserExecute | Self::Kernel)
    }

    const fn user(self) -> bool {
        !matches!(self, Self::Kernel)
    }

    const fn global(self) -> bool {
        !self.user()
    }
}

#[bitsize(64)]
#[derive(Copy, Clone, DebugBits)]
pub struct Sv39PageTableEntry {
    valid: bool,
    read: bool,
    write: bool,
    execute: bool,
    user: bool,
    global: bool,
    _unused: u4,
    ppn0: u9,
    ppn1: u9,
    ppn2: u26,
    _reserved: u10,
}

impl Sv39PageTableEntry {
    const fn zero() -> Self {
        Self { value: 0 }
    }

    /// Is this a leaf node(i.e., does not link to a page table)?
    pub fn is_leaf(&self) -> bool {
        self.read() | self.write() | self.execute()
    }

    /// Does this link to another page table?
    pub fn is_branch(&self) -> bool {
        !self.is_leaf()
    }

    pub fn physical_address(&self) -> usize {
        assert!(self.valid());
        let ppn2: u64 = self.ppn2().into();
        let ppn1: u64 = self.ppn1().into();
        let ppn0: u64 = self.ppn0().into();
        let paddr = (ppn2 << (18 + 12)) | (ppn1 << (9 + 12)) | (ppn0 << (12));
        paddr
            .try_into()
            .expect("Failed to convert u64 to usize - check architecture")
    }
}

#[repr(align(4096))]
#[derive(Debug)]
pub struct Sv39PageTable {
    pub entries: [Sv39PageTableEntry; ENTRIES_IN_PAGE_TABLE],
}

impl Sv39PageTable {
    const fn new() -> Self {
        Self {
            entries: [Sv39PageTableEntry::zero(); ENTRIES_IN_PAGE_TABLE],
        }
    }

    unsafe fn mut_from_addr<'a>(address: usize) -> &'a mut Self {
        unsafe { &mut *(address as *mut Self) }
    }

    fn entry(&self, index: usize) -> Sv39PageTableEntry {
        assert!(index < ENTRIES_IN_PAGE_TABLE);
        self.entries[index]
    }

    fn set_entry(&mut self, index: usize, entry: Sv39PageTableEntry) -> Sv39PageTableEntry {
        assert!(index < ENTRIES_IN_PAGE_TABLE);
        self.entries[index] = entry;
        entry
    }
}

#[bitsize(64)]
#[derive(TryFromBits, Copy, Clone, DebugBits)]
pub struct Sv39VirtualAddress {
    page_offset: u12,
    vpn0: u9,
    vpn1: u9,
    vpn2: u9,
    _ignored: u25,
}

impl Sv39VirtualAddress {
    fn is_page_aligned(&self) -> bool {
        usize::from(*self) & (PAGE_SIZE - 1) == 0
    }

    fn offset(&self, offset: isize) -> KernelResult<Self> {
        let val: usize = (*self).into();
        // Saturating is OK here, because `try_from` will error out if it's actually saturated
        Self::try_from(val.saturating_add_signed(offset))
    }
}

impl TryFrom<usize> for Sv39VirtualAddress {
    type Error = KernelError;
    fn try_from(other: usize) -> KernelResult<Self> {
        if other > MAX_VIRTUAL_ADDRESS {
            return Err(KernelError::InvalidVirtualAddress(other));
        }
        Ok(Self::try_from(other as u64)
            .expect("Expected address to be valid since we already checked it"))
    }
}

impl From<Sv39VirtualAddress> for usize {
    fn from(other: Sv39VirtualAddress) -> Self {
        other.value as Self
    }
}

#[bitsize(64)]
#[derive(TryFromBits, Copy, Clone, DebugBits, PartialEq, Eq)]
pub struct Sv39PhysicalAddress {
    page_offset: u12,
    ppn0: u9,
    ppn1: u9,
    ppn2: u26,
    _ignored: u8,
}

impl Sv39PhysicalAddress {
    fn is_page_aligned(&self) -> bool {
        usize::from(*self) & (PAGE_SIZE - 1) == 0
    }

    fn offset(&self, offset: isize) -> KernelResult<Self> {
        let val: usize = (*self).into();
        // Saturating is OK here, because `try_from` will error out if it's actually saturated
        Self::try_from(val.saturating_add_signed(offset))
    }
}

impl TryFrom<usize> for Sv39PhysicalAddress {
    type Error = KernelError;
    fn try_from(other: usize) -> KernelResult<Self> {
        if other > MAX_PHYSICAL_ADDRESS {
            return Err(KernelError::InvalidPhysicalAddress(other));
        }
        Ok(Self::try_from(other as u64)
            .expect("Expected address to be valid since we already checked it"))
    }
}

impl From<Sv39PhysicalAddress> for usize {
    fn from(other: Sv39PhysicalAddress) -> Self {
        other.value as Self
    }
}

/// Start the MMU
pub fn init_mmu(pmo: isize) -> KernelResult<()> {
    // Enable page protections
    // Necessary to prevent fault for mret
    // Don't fully understand this yet 😅
    unsafe {
        riscv::register::pmpcfg0::set_pmp(
            0,
            riscv::register::Range::NAPOT,
            riscv::register::Permission::RWX,
            false,
        );
        riscv::register::pmpaddr0::write(0xFFFF_FFFF_FFFF_FFFF);
    }

    // Fence
    riscv::asm::sfence_vma_all();

    // Set PMO
    critical_section::with(|cs| {
        PHYSICAL_MEMORY_OFFSET.borrow(cs).set(pmo);
    });

    Ok(())
}

/// Initialize paging and all that jazz
pub fn init_page_tables(pmo: isize) -> KernelResult<()> {
    self_test();

    let page_table_paddr = critical_section::with(|cs| {
        // We have to make sure we're actually getting the table address here
        let table = ROOT_PAGE_TABLE.borrow(cs);
        let table: &Sv39PageTable = &table.borrow();
        ptr::from_ref(table) as usize
    });

    set_root_page_table(0, page_table_paddr.try_into()?);

    critical_section::with(|cs| {
        let mut root_page_table = ROOT_PAGE_TABLE.borrow_ref_mut(cs);
        map_kernel_space_with_pmo_offset(&mut root_page_table, pmo)
    })?;

    Ok(())
}

/// Map kernel space onto page table
pub fn map_kernel_space(table: &mut Sv39PageTable) -> KernelResult<()> {
    map_kernel_space_with_pmo_offset(table, 0)
}

fn map_kernel_space_with_pmo_offset(
    table: &mut Sv39PageTable,
    pmo_offset: isize,
) -> KernelResult<()> {
    let pmo = critical_section::with(|cs| PHYSICAL_MEMORY_OFFSET.borrow(cs).get());
    unsafe {
        let kernel_start_addr = ptr::from_ref(&kernel_start) as usize;
        assert!((kernel_start_addr as isize) >= pmo);
        map_range(
            table,
            // The GOT table was offshifted by PMO in asm, so we have to shift the virtual pages
            // back
            Sv39VirtualAddress::try_from(
                kernel_start_addr.checked_add_signed(-pmo_offset).unwrap(),
            )?,
            Sv39PhysicalAddress::try_from(kernel_start_addr.checked_add_signed(pmo).unwrap())?,
            PageType::Kernel,
            ptr::from_ref(&table_heap_top) as usize - kernel_start_addr,
        )?;
    }

    unsafe {
        let stack_bottom_addr = ptr::from_ref(&stack_bottom) as usize;
        let stack_top_addr = ptr::from_ref(&stack_top) as usize;
        map_range(
            table,
            Sv39VirtualAddress::try_from(
                stack_bottom_addr.checked_add_signed(-pmo_offset).unwrap(),
            )?,
            Sv39PhysicalAddress::try_from(stack_bottom_addr.checked_add_signed(pmo).unwrap())?,
            PageType::Kernel,
            stack_top_addr - stack_bottom_addr,
        )?;
    }

    Ok(())
}

/// Set the root page table address
pub fn set_root_page_table(asid: u16, paddr: Sv39PhysicalAddress) {
    // PPN is 4k-aligned, and we don't store the trailing zeroes
    let paddr = usize::from(paddr) >> 12;
    unsafe {
        riscv::register::satp::set(riscv::register::satp::Mode::Sv39, asid as usize, paddr);
    }
}

/// Get physical address from kernel space virtual address
pub fn ks_vaddr_to_paddr(vaddr: usize) -> KernelResult<Sv39PhysicalAddress> {
    critical_section::with(|cs| {
        let root_page_table = ROOT_PAGE_TABLE.borrow_ref_mut(cs);
        vaddr_to_paddr(&root_page_table, vaddr)
    })?
    .ok_or(KernelError::NotMapped(vaddr))
}

/// Get physical mapping by walking page table
pub fn vaddr_to_paddr(
    mut table: &Sv39PageTable,
    vaddr: usize,
) -> KernelResult<Option<Sv39PhysicalAddress>> {
    let vaddr = Sv39VirtualAddress::try_from(vaddr)?;
    let pmo = critical_section::with(|cs| PHYSICAL_MEMORY_OFFSET.borrow(cs).get());

    // Walk page tables
    for step in 0..=2 {
        let index = u16::from([vaddr.vpn2(), vaddr.vpn1(), vaddr.vpn0()][step]) as usize;

        let entry = table.entry(index);

        if !entry.valid() {
            return Ok(None);
        }

        if entry.is_leaf() {
            return Ok(Some(entry.physical_address().try_into()?));
        }

        table = unsafe {
            Sv39PageTable::mut_from_addr(entry.physical_address().checked_add_signed(-pmo).unwrap())
        };
    }
    panic!("Walked right off the table!");
}

/// Map a memory-mapped device to kernel space
pub fn map_device(phys_address: usize, size: usize) -> KernelResult<usize> {
    Serial::new().write_str("<map_device>\n")?;
    assert!(size >= 0x1000);
    let virt_address = phys_address;
    critical_section::with(|cs| {
        let mut root_page_table = ROOT_PAGE_TABLE.borrow_ref_mut(cs);
        map_range(
            &mut root_page_table,
            phys_address.try_into()?,
            virt_address.try_into()?,
            PageType::Kernel,
            size,
        )
    })?;
    Ok(virt_address)
}

/// Map a range of addresses
pub fn map_range(
    table: &mut Sv39PageTable,
    mut vaddr: Sv39VirtualAddress,
    mut paddr: Sv39PhysicalAddress,
    page_type: PageType,
    size: usize,
) -> KernelResult<()> {
    println!("<map_range table={:?}>", table as *mut Sv39PageTable);
    assert!(size > 0);
    if !vaddr.is_page_aligned() {
        return Err(KernelError::AddressNotPageAligned(usize::from(vaddr)));
    }
    if !paddr.is_page_aligned() {
        return Err(KernelError::AddressNotPageAligned(usize::from(paddr)));
    }

    if size % PAGE_SIZE != 0 {
        return Err(KernelError::SizeMisaligned(size));
    }

    for _ in 0..size / PAGE_SIZE {
        map_page(table, vaddr, paddr, page_type)?;
        assert_eq!(vaddr_to_paddr(table, vaddr.into())?.unwrap(), paddr);

        vaddr = vaddr.offset(PAGE_SIZE as isize)?;
        paddr = paddr.offset(PAGE_SIZE as isize)?;
    }

    Ok(())
}

/// Map a single virtual page to a physical page
pub fn map_page(
    mut table: &mut Sv39PageTable,
    vaddr: Sv39VirtualAddress,
    paddr: Sv39PhysicalAddress,
    page_type: PageType,
) -> KernelResult<()> {
    println!(
        "map@{:?}: v{:08x}-> p{:08x}",
        table as *mut Sv39PageTable,
        usize::from(vaddr),
        usize::from(paddr)
    );
    if !vaddr.is_page_aligned() {
        return Err(KernelError::AddressNotPageAligned(usize::from(vaddr)));
    }
    if !paddr.is_page_aligned() {
        return Err(KernelError::AddressNotPageAligned(usize::from(paddr)));
    }

    let pmo = critical_section::with(|cs| PHYSICAL_MEMORY_OFFSET.borrow(cs).get());

    // Walk page tables
    for step in 0..2 {
        let index = u16::from([vaddr.vpn2(), vaddr.vpn1()][step]) as usize;

        let mut entry = table.entry(index);
        if !entry.valid() {
            let addr = (zalloc::<Page<PAGE_SIZE>>().leak() as usize)
                .checked_add_signed(pmo)
                .unwrap();
            let phys = Sv39PhysicalAddress::try_from(addr)?;
            entry = table.set_entry(
                index,
                Sv39PageTableEntry::new(
                    true,
                    false,
                    false,
                    false,
                    page_type.user(),
                    page_type.global(),
                    Default::default(),
                    phys.ppn0(),
                    phys.ppn1(),
                    phys.ppn2(),
                ),
            );
            assert_eq!(addr, entry.physical_address())
        }
        if entry.is_leaf() {
            panic!("Unexpected leaf node in page table! (step {step}): {entry:?}");
        }

        table = unsafe {
            Sv39PageTable::mut_from_addr(
                entry
                    .physical_address()
                    .checked_add_signed(-pmo)
                    .expect("Table address translation overflow!!"),
            )
        };
    }

    let index = u16::from(vaddr.vpn0()) as usize;
    if table.entry(index).valid() {
        panic!("Table already mapped!");
    }

    table.set_entry(
        index,
        Sv39PageTableEntry::new(
            true,
            page_type.read(),
            page_type.write(),
            page_type.execute(),
            page_type.user(),
            page_type.global(),
            Default::default(),
            paddr.ppn0(),
            paddr.ppn1(),
            paddr.ppn2(),
        ),
    );

    Ok(())
}

/// A generic page of any size
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Page<const SIZE: usize>([u8; SIZE]);

/// A self-deallocating page allocation
#[derive(Debug)]
pub struct PageAllocation<T: ?Sized> {
    address: Option<*mut T>,
    num_pages: usize,
}

impl<T: ?Sized> PageAllocation<T> {
    /// Leak allocation without deallocating
    pub fn leak(mut self) -> *mut T {
        // Can't leak twice
        self.address.take().unwrap()
    }

    /// Return as const pointer
    pub fn as_const_ptr(&self) -> *const T {
        self.address.unwrap() as *const T
    }

    /// Return as mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.address.unwrap()
    }

    /// Return raw address
    pub fn addr(&self) -> usize {
        self.address.unwrap() as *const () as usize
    }

    /// Get number of pages
    pub fn num_pages(&self) -> usize {
        self.num_pages
    }
}

impl<T: ?Sized> AsRef<T> for PageAllocation<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.address.unwrap() }
    }
}

impl<T: ?Sized> AsMut<T> for PageAllocation<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.address.unwrap() }
    }
}

impl<T: ?Sized> Drop for PageAllocation<T> {
    fn drop(&mut self) {
        if let Some(address) = self.address {
            let records = unsafe { ptr::addr_of_mut!(table_heap_bottom) };
            let top = unsafe { ptr::from_ref(&table_heap_top) };
            let first_page_address = records as usize + PAGE_SIZE;
            let heap_size = (top as usize - first_page_address) / PAGE_SIZE;

            unsafe {
                (*records).deallocate(first_page_address as *const c_void, heap_size, address);
            }
        }
    }
}

/// Allocate and zero a new T, with page-grain allocation
pub fn zalloc<T>() -> PageAllocation<T> {
    let records = unsafe { ptr::addr_of_mut!(table_heap_bottom) };
    let top = unsafe { ptr::from_ref(&table_heap_top) };
    let first_page_address = records as usize + PAGE_SIZE;
    let heap_size = (top as usize - first_page_address) / PAGE_SIZE;

    let allocation = unsafe { (*records).allocate(first_page_address as *const c_void, heap_size) };

    zero_out(allocation.0, PAGE_SIZE * allocation.1);

    PageAllocation {
        address: Some(allocation.0),
        num_pages: allocation.1,
    }
}

/// Allocate and zero a new `[T]`, with page-grain allocation
pub fn zalloc_slice<T>(num_pages: usize) -> PageAllocation<[T]> {
    let records = unsafe { ptr::addr_of_mut!(table_heap_bottom) };
    let top = unsafe { ptr::from_ref(&table_heap_top) };
    let first_page_address = records as usize + PAGE_SIZE;
    let heap_size = (top as usize - first_page_address) / PAGE_SIZE;

    let address = unsafe {
        (*records).allocate_slice(first_page_address as *const c_void, heap_size, num_pages)
    };
    zero_out(address, num_pages * PAGE_SIZE);

    PageAllocation {
        address: Some(address),
        num_pages,
    }
}

// Zero-out bytes
fn zero_out<T: ?Sized>(address: *const T, size: usize) {
    let address = address as *const () as usize;
    assert!(size > 0);
    assert!(address > 0);
    for byte in address..(address + size) {
        let byte = byte as *mut u8;
        unsafe {
            *byte = 0;
        }
    }
}

fn self_test() {
    {
        let paddr: u64 = 0xdeadbeef;
        let paddr = Sv39PhysicalAddress::try_from(paddr).unwrap();
        assert_eq!(paddr.page_offset(), u12::new(0xeef));
        assert_eq!(paddr.ppn0(), u9::new(0xdb));
        assert_eq!(paddr.value, 0xdeadbeef);
    }
}
