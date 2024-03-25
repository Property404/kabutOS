#![no_std]
#![no_main]
mod shell;
use core::time::Duration;
use kanto::{prelude::*, sys};

#[no_mangle]
extern "C" fn main() {
    println!("[dratinit] starting forks!");

    if let Some(pid) = sys::fork().unwrap() {
        sys::wait_pid(pid).unwrap();
    } else {
        shell::shell();
        sys::exit_ok().unwrap();
    }

    println!("[dratinit] Entering eternal loop!");

    // Don't exit init
    loop {
        println!("...");
        let _ = sys::sleep(Duration::from_millis(1000));
    }
}
