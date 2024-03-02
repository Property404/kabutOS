//! A page-grain allocator for embedded systems
#![cfg_attr(not(test), no_std)]
#[warn(missing_docs)]
mod record;
use core::{ffi::c_void, mem, ptr};
use record::Record;

const fn align_up<const SIZE: usize>(val: usize) -> usize {
    let mut rv = align_down::<SIZE>(val);
    if val % SIZE != 0 {
        rv += SIZE;
    }
    rv
}

const fn align_down<const SIZE: usize>(val: usize) -> usize {
    assert!(SIZE.is_power_of_two());
    SIZE * (val / SIZE)
}

/// A single book-keeping page for an upward-growing page allocator
///
/// This should transparently point to the beginning of the page heap.
#[repr(transparent)]
#[derive(Debug)]
pub struct RecordsPage<const PAGE_SIZE: usize>([u8; PAGE_SIZE]);

impl<const PAGE_SIZE: usize> RecordsPage<PAGE_SIZE> {
    const RECORD_SIZE_IN_BITS: usize = 2;
    const NUM_RECORDS_IN_BYTE: usize = 8 / Self::RECORD_SIZE_IN_BITS;
    const NUM_RECORDS_IN_PAGE: usize = {
        assert!(PAGE_SIZE.is_power_of_two());
        PAGE_SIZE * Self::NUM_RECORDS_IN_BYTE
    };

    fn get_record(&self, index: usize) -> Record {
        let byte_index = index / Self::NUM_RECORDS_IN_BYTE;
        Record::from_byte(self.0[byte_index])[index % Self::NUM_RECORDS_IN_BYTE]
    }

    fn set_record(&mut self, index: usize, record: Record) {
        let byte_index = index / Self::NUM_RECORDS_IN_BYTE;
        let mut records = Record::from_byte(self.0[byte_index]);
        records[index % Self::NUM_RECORDS_IN_BYTE] = record;
        self.0[byte_index] = Record::to_byte(records);
    }

    fn is_taken(&mut self, index: usize) -> bool {
        self.get_record(index).taken
    }

    fn is_last(&mut self, index: usize) -> bool {
        self.get_record(index).last
    }

    fn set_taken(&mut self, index: usize, taken: bool) {
        let mut record = self.get_record(index);
        record.taken = taken;
        self.set_record(index, record);
    }

    fn set_last(&mut self, index: usize, last: bool) {
        let mut record = self.get_record(index);
        record.last = last;
        self.set_record(index, record);
    }

    /// Deallocate a region of memory
    ///
    /// # Returns
    /// The number of pages deallocated
    pub fn deallocate<T: ?Sized>(
        &mut self,
        heap_start: *const c_void,
        heap_size: usize,
        address: *const T,
    ) -> usize {
        let address = address as *const () as usize;
        assert_eq!(
            heap_start as usize & (PAGE_SIZE - 1),
            0,
            "`heap_start` not page-aligned"
        );
        assert_eq!(address & (PAGE_SIZE - 1), 0, "Address not page-aligned");
        assert!((address) >= heap_start as usize, "Address below range");
        assert!(
            (address) < (heap_start as usize + heap_size * PAGE_SIZE),
            "Address above range"
        );
        assert_ne!(heap_size, 0);

        let mut record_index = address
            .checked_sub(heap_start as usize)
            .expect("Address out-of-range")
            / PAGE_SIZE;

        // Previous free page is no longer last
        if record_index > 0 && !self.is_taken(record_index - 1) && self.is_last(record_index - 1) {
            self.set_last(record_index - 1, false)
        }

        let mut num_deallocated = 0;
        loop {
            assert!(self.is_taken(record_index), "Double free!");

            self.set_taken(record_index, false);
            num_deallocated += 1;

            if self.is_last(record_index) {
                if record_index < heap_size && !self.is_taken(record_index + 1) {
                    self.set_last(record_index, false);
                }
                break;
            }

            record_index += 1;
        }

        num_deallocated
    }

    fn allocate_inner(
        &mut self,
        heap_start: *const c_void,
        heap_size: usize,
        num_pages: usize,
    ) -> *mut () {
        assert_eq!(
            heap_start as usize & (PAGE_SIZE - 1),
            0,
            "Heap_start not page-aligned"
        );
        assert_ne!(heap_size, 0);
        assert_ne!(num_pages, 0);
        let mut count = 0;

        for record_index in 0..Self::NUM_RECORDS_IN_PAGE {
            if record_index >= heap_size {
                panic!("Heap overflow!");
            }
            let record = self.get_record(record_index);

            if record.taken {
                count = 0;
                continue;
            }
            count += 1;

            if count >= num_pages {
                let page_start = record_index + 1 - count;

                // Set previous record to last
                if page_start > 0 {
                    self.set_last(page_start - 1, true);
                }

                // Set all records to taken
                for i in page_start..(page_start + count) {
                    self.set_taken(i, true);
                }

                // Last here marks end of allocation
                self.set_last(page_start + count - 1, true);

                // And return start of page
                return ((heap_start as usize) + page_start * PAGE_SIZE) as *mut ();
            }

            // Nevermind, we can't use this
            if record.last {
                count = 0;
            }
        }

        panic!("Heap overflow(book-keeping overrun)!");
    }

    /// Allocate some pages
    ///
    /// # Arguments
    /// `heap_start` - The start of the page heap
    /// `heap_size` - The size of the page heap IN PAGES
    ///
    /// # Panics
    /// * If `heap_size` is zero
    /// * If size of T is zero
    /// * If there is a heap overflow
    /// * `heap_start` is not page-aligned
    ///
    /// # Returns
    /// The address of the first allocated page
    pub fn allocate<T>(&mut self, heap_start: *const c_void, heap_size: usize) -> *mut T {
        let num_pages = align_up::<PAGE_SIZE>(mem::size_of::<T>());
        self.allocate_inner(heap_start, heap_size, num_pages) as *mut T
    }

    /// Same as [allocate], but for slices. This is typically used to dynamically allocate multiple
    /// pages
    ///
    /// # Panics
    /// If T is not page-sized (might be lifted in future)
    /// If any of the invariants of [allocate] are not met
    pub fn allocate_slice<T>(
        &mut self,
        heap_start: *const c_void,
        heap_size: usize,
        num_pages: usize,
    ) -> *mut [T] {
        assert_eq!(mem::size_of::<T>(), PAGE_SIZE);

        let ptr = self.allocate_inner(heap_start, heap_size, num_pages) as *mut T;
        let size = num_pages * PAGE_SIZE;
        ptr::slice_from_raw_parts_mut(ptr, size)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::ptr::null;

    const PAGE_SIZE: usize = 2048;
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Page([u8; PAGE_SIZE]);

    #[test]
    fn allocate() {
        const PAGES: usize = 1024;

        let mut records_page = RecordsPage([Default::default(); PAGE_SIZE]);

        let mut sum = 0;
        for num_pages_to_allocate in [1, 5, 3, 4, 100, 1] {
            assert_eq!(
                records_page.allocate_slice::<Page>(null(), PAGES, num_pages_to_allocate)
                    as *const () as usize,
                PAGE_SIZE * sum
            );
            sum += num_pages_to_allocate;
        }
    }

    #[test]
    fn alloc_dealloc() {
        const PAGES: usize = 1024;

        let mut records_page = RecordsPage([Default::default(); PAGE_SIZE]);

        let tv = [1, 5, 3, 4, 100, 1];
        let mut addresses = Vec::new();

        let mut sum = 0;
        for num_pages_to_allocate in tv {
            let address = records_page.allocate_slice::<Page>(null(), PAGES, num_pages_to_allocate);
            addresses.push(address);
            assert_eq!(address as *const () as usize, PAGE_SIZE * sum);
            sum += num_pages_to_allocate;
        }

        // Now deallocate everything
        for (index, address) in addresses.iter().enumerate().rev() {
            assert_eq!(records_page.deallocate(null(), PAGES, *address), tv[index]);
        }
    }
}
