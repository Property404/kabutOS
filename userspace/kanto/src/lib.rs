//! KabutOS userland library. Named after the region in Pokemon generation I

#![no_std]
#![warn(missing_docs)]
global_asm!(include_str!("asm.S"));
global_asm!(include_str!("crt.S"));

#[doc(hidden)]
pub mod serial;
pub mod sys;
pub mod prelude {
    //! Userspace prelude
    pub use crate::{print, println};
}

use core::arch::global_asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}
