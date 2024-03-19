#![no_std]
#![no_main]
use kanto::{prelude::*, sys};

const TESTS: [fn(); 1] = [fork_and_wait];

fn fork_and_wait() {
    let pid = sys::fork().unwrap();
    if let Some(pid) = pid {
        sys::wait_pid(pid).unwrap();
    } else {
        sys::exit_ok().unwrap();
    }
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
