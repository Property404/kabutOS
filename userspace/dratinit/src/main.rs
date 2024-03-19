#![no_std]
#![no_main]
use core::time::Duration;
use kanto::{prelude::*, sys};

fn shell() {
    println!("Wooh! Shell\n");
    sys::exit_ok().unwrap();
}

#[no_mangle]
extern "C" fn main() {
    println!("[dratinit] starting forks!");

    let mut pids = Vec::new();

    for _ in 0..4 {
        if let Some(pid) = sys::fork().unwrap() {
            pids.push(pid);
        } else {
            shell();
        }
    }

    for pid in pids {
        sys::wait_pid(pid).unwrap();
    }

    println!("[dratinit] Entering eternal loop!");

    // Don't exit init
    loop {
        println!("...");
        let _ = sys::sleep(Duration::from_millis(1000));
    }
}
