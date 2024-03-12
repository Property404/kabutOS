#![no_std]
#![no_main]
use kanto::{prelude::*, sys};

#[no_mangle]
extern "C" fn main() {
    let pid = sys::get_pid();
    println!("Hello, Sweetie, from pid {pid}!");
    for _ in 0..4 {
        if pid % 2 == 0 {
            println!("Howdy");
        } else {
            println!("Hmm!");
        }
    }
}
