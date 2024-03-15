#![no_std]
#![no_main]
use kanto::{prelude::*, sys};

#[no_mangle]
extern "C" fn main() {
    let child_pid = sys::fork().unwrap();
    let pid = sys::get_pid().unwrap();

    if let Some(child_pid) = child_pid {
        sys::wait_pid(child_pid).unwrap();
    }

    let speaker = if child_pid.is_none() {
        "child"
    } else {
        "mother"
    };
    println!("{speaker}: Hello, Sweetie, from pid {pid}!");

    for _ in 0..4 {
        if u16::from(pid) % 2 == 0 {
            println!("Howdy");
        } else {
            println!("Hmm!");
        }
    }

    // Don't exit init
    if child_pid.is_some() {
        #[allow(clippy::empty_loop)]
        loop {}
    }
}
