//! MMU and paging setup
//!
//! Most of this is based off <https://osblog.stephenmarz.com/ch3.2.html>
use crate::serial::Serial;
use bilge::prelude::*;
use core::{cell::RefCell, ffi::c_void, fmt::Write};
use critical_section::Mutex;

extern "C" {
    static heap_bottom: c_void;
    static heap_top: c_void;
    static kernel_start: c_void;
    static stack_bottom: c_void;
    static stack_top: c_void;
    fn enter_supervisor_mode();
}

const PAGE_SIZE: usize = 4096;
const MAX_VIRTUAL_ADDRESS: usize = (1 << 39) - 1;
const MAX_PHYSICAL_ADDRESS: usize = (1 << 56) - 1;
const ENTRIES_IN_PAGE_TABLE: usize = 512;

#[bitsize(64)]
#[derive(Copy, Clone, DebugBits)]
pub struct Sv39PageTableEntry {
    valid: bool,
    read: bool,
    write: bool,
    execute: bool,
    _unused: u6,
    ppn0: u9,
    ppn1: u9,
    ppn2: u26,
    _reserved: u10,
}

impl Sv39PageTableEntry {
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

pub struct Sv39PageTable {
    pub entries: [Sv39PageTableEntry; ENTRIES_IN_PAGE_TABLE],
}

impl Sv39PageTable {
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
#[derive(TryFromBits)]
pub struct Sv39VirtualAddress {
    page_offset: u12,
    vpn0: u9,
    vpn1: u9,
    vpn2: u9,
    _ignored: u25,
}

#[bitsize(64)]
#[derive(TryFromBits)]
pub struct Sv39PhysicalAddress {
    page_offset: u12,
    ppn0: u9,
    ppn1: u9,
    ppn2: u26,
    _ignored: u8,
}

/// Initialize paging and all that jazz
pub fn init_mmu() {
    self_test();

    set_root_page_table(zalloc_page());

    let (bottom, top) = unsafe {
        (
            &kernel_start as *const c_void as usize,
            &heap_top as *const c_void as usize,
        )
    };
    for page in (bottom..top).step_by(4096) {
        map_vaddr_to_paddr(page, page);
        assert_eq!(page, vaddr_to_paddr(page).unwrap());
    }

    for page in unsafe {
        (&stack_bottom as *const c_void as usize..&stack_top as *const c_void as usize)
            .step_by(4096)
    } {
        map_vaddr_to_paddr(page, page);
        assert_eq!(page, vaddr_to_paddr(page).unwrap());
    }
    map_vaddr_to_paddr(0x1000_0000, 0x1000_0000);
    assert_eq!(0x1000_0000, vaddr_to_paddr(0x1000_0000).unwrap());

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
    unsafe { riscv::asm::sfence_vma_all() };

    // Enter supervisor mode
    unsafe { enter_supervisor_mode() }
}

/// Set the root page table address
fn set_root_page_table(paddr: usize) {
    assert!(paddr < MAX_PHYSICAL_ADDRESS);
    assert_eq!(
        paddr & (PAGE_SIZE - 1),
        0,
        "Physical address not 4k-aligned"
    );

    let _ = writeln!(Serial::new(), "Creating root page table @ {paddr:08x}");

    // PPN is 4k-aligned, and we don't store the trailing zeroes
    let paddr = (paddr) >> 12;
    riscv::register::satp::write((8 << 60) | paddr);
}

/// Get physical mapping by walking page table
#[must_use]
pub fn vaddr_to_paddr(vaddr: usize) -> Option<usize> {
    let mut table =
        unsafe { Sv39PageTable::mut_from_addr(riscv::register::satp::read().ppn() << 12) };
    let vaddr = Sv39VirtualAddress::try_from(vaddr as u64).expect("Invalid Address");

    // Walk page tables
    for step in 0..=2 {
        let index = u16::from([vaddr.vpn2(), vaddr.vpn1(), vaddr.vpn0()][step]) as usize;

        let entry = table.entry(index);

        if !entry.valid() {
            return None;
        }

        if entry.is_leaf() {
            return Some(entry.physical_address());
        }

        table = unsafe { Sv39PageTable::mut_from_addr(entry.physical_address()) };
    }
    panic!("Walked right off the table!");
}

/// Map a virtual address to a physical address, in 4k blocks
pub fn map_vaddr_to_paddr(vaddr: usize, paddr: usize) {
    assert_eq!(vaddr & (PAGE_SIZE - 1), 0, "Virtual address not 4k-aligned");
    assert_eq!(
        paddr & (PAGE_SIZE - 1),
        0,
        "Physical address not 4k-aligned"
    );
    assert!(vaddr < MAX_VIRTUAL_ADDRESS);
    assert!(paddr < MAX_PHYSICAL_ADDRESS);
    let vaddr = Sv39VirtualAddress::try_from(vaddr as u64).expect("Invalid Address");
    let paddr = Sv39PhysicalAddress::try_from(paddr as u64).expect("Invalid Address");

    // TODO: The page tables are shared memory, and therefore this needs to be guarded with a critical
    // section. Any access to the page tables needs to be guarded. But doing that efficiently
    // seems..complicated?

    let mut table =
        unsafe { Sv39PageTable::mut_from_addr(riscv::register::satp::read().ppn() << 12) };

    // Walk page tables
    for step in 0..2 {
        let index = u16::from([vaddr.vpn2(), vaddr.vpn1()][step]) as usize;

        let mut entry = table.entry(index);
        if !entry.valid() {
            let addr = zalloc_page();
            let phys = Sv39PhysicalAddress::try_from(addr as u64).expect("Bad physical address");
            entry = table.set_entry(
                index,
                Sv39PageTableEntry::new(
                    true,
                    false,
                    false,
                    false,
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

        table = unsafe { Sv39PageTable::mut_from_addr(entry.physical_address()) };
    }

    let index = u16::from(vaddr.vpn0()) as usize;
    if table.entry(index).valid() {
        panic!("Table already mapped!");
    }

    table.set_entry(
        index,
        Sv39PageTableEntry::new(
            true,
            true,
            true,
            true,
            Default::default(),
            paddr.ppn0(),
            paddr.ppn1(),
            paddr.ppn2(),
        ),
    );
}

/// Get a new 4k-aligned page
fn kalloc_page() -> usize {
    static HEAP: Mutex<RefCell<Option<usize>>> = Mutex::new(RefCell::new(None));

    critical_section::with(|cs| {
        let mut heap = HEAP.borrow_ref_mut(cs);
        if let Some(new_page) = *heap {
            if new_page >= unsafe { &heap_top as *const c_void as usize } {
                panic!("Heap overflow!");
            }
            *heap = Some(new_page + PAGE_SIZE);
            new_page
        } else {
            let bottom = unsafe { &heap_bottom as *const c_void as usize };
            *heap = Some(bottom + PAGE_SIZE);
            bottom
        }
    })
}

/// Get a new ZEROED 4k-aligned page
fn zalloc_page() -> usize {
    let page: usize = kalloc_page();
    {
        for byte in page..(page + PAGE_SIZE) {
            let byte = byte as *mut u8;
            unsafe {
                *byte = 0;
            }
        }
    }

    page
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
