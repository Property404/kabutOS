#![no_std]
#![no_main]
use kanto::{prelude::*, sys};

#[no_mangle]
extern "C" fn main() {
    let child_pid = sys::fork().unwrap();
    let pid = sys::get_pid().unwrap();

    if child_pid != 0 {
        sys::wait_pid(child_pid).unwrap();
    }

    let speaker = if child_pid == 0 { "child" } else { "mother" };
    println!("{speaker}: Hello, Sweetie, from pid {pid}!");

    for _ in 0..4 {
        if pid % 2 == 0 {
            println!("Howdy");
        } else {
            println!("Hmm!");
        }
    }

    // Don't exit init
    if child_pid != 0 {
        #[allow(clippy::empty_loop)]
        loop {}
    }
}
