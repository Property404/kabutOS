//! KabutOS userland library. Named after the region in Pokemon generation I

#![no_std]
#![warn(missing_docs)]
global_asm!(include_str!("asm.S"));
global_asm!(include_str!("crt.S"));

mod allocator;
extern crate alloc;

#[doc(hidden)]
pub mod serial;
pub mod sys;
pub mod prelude {
    //! Userspace prelude
    pub use crate::{print, println};
    pub use alloc::{boxed::Box, string::String, vec::Vec};
}

pub use krabby_abi as abi;

use core::arch::global_asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

// Automatic exit after program ends
#[no_mangle]
extern "C" fn _exit() {
    sys::exit_ok().unwrap();
    let pid = sys::get_pid().unwrap();
    panic!("Process {pid} failed to exit");
}
