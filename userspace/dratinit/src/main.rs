#![no_std]
#![no_main]
use core::time::Duration;
use kanto::{prelude::*, sys};

fn shell() {
    println!("Wooh! Shell\n");
    print!("$ ");
    loop {
        let c = sys::getc().unwrap();
        if c == '\r' {
            sys::puts("\n$ ").unwrap();
        } else {
            sys::putc(c).unwrap();
        }
    }
}

#[no_mangle]
extern "C" fn main() {
    println!("[dratinit] starting forks!");

    if let Some(pid) = sys::fork().unwrap() {
        sys::wait_pid(pid).unwrap();
    } else {
        shell();
        sys::exit_ok().unwrap();
    }

    println!("[dratinit] Entering eternal loop!");

    // Don't exit init
    loop {
        println!("...");
        let _ = sys::sleep(Duration::from_millis(1000));
    }
}
