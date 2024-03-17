#![no_std]
#![no_main]
use core::time::Duration;
use kanto::{prelude::*, sys};

fn shell() {
    println!("Wooh! Shell\n");
    sys::exit().unwrap();
}

#[no_mangle]
extern "C" fn main() {
    println!("[dratinit] starting forks!");

    for _ in 0..4 {
        if let Some(cpid) = sys::fork().unwrap() {
            sys::wait_pid(cpid).unwrap();
        } else {
            shell();
        }
    }

    println!("[dratinit] Entering eternal loop!");

    // Don't exit init
    loop {
        println!("...");
        let _ = sys::sleep(Duration::from_millis(1000));
    }
}
