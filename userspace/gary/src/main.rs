#![no_std]
#![no_main]
use core::sync::atomic::{AtomicU32, Ordering};
use kanto::{prelude::*, sys};

const TESTS: &[fn()] = &[fork_and_wait, static_vars, allocate_multiple_pages];

fn fork_and_wait() {
    let pid = sys::fork().unwrap();
    if let Some(pid) = pid {
        sys::wait_pid(pid).unwrap();
    } else {
        sys::exit_ok().unwrap();
    }
}

// This tests a bug that causes a StorePageFault when writing to a static variable.
// I believe this to be a result of the data section being marked execute only
fn static_vars() {
    static VAL: core::sync::atomic::AtomicU32 = AtomicU32::new(40);
    VAL.fetch_add(1, Ordering::Relaxed);
}

// This makes sure the global allocator can allocate more than a single page (4K)
fn allocate_multiple_pages() {
    const PAGE_SIZE: usize = 0x1000;
    let mut vec = Vec::<u8>::new();
    for i in 0..PAGE_SIZE {
        vec.push((i % 5).try_into().unwrap());
    }
    assert_eq!(vec.len(), PAGE_SIZE);
}

#[no_mangle]
extern "C" fn main() {
    for test in TESTS {
        println!("[gary: running test]");
        if let Some(pid) = sys::fork().unwrap() {
            sys::wait_pid(pid).unwrap();
        } else {
            test();
            sys::exit_ok().unwrap();
        }
    }

    sys::power_off().unwrap();
}
