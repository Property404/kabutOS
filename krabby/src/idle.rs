use crate::frame;
#[allow(unused_imports)]
use riscv::{asm::wfi, register::sstatus};

/// Switch to idle "process" and return its PC
// Maybe we should factor out this functionality along with Process::switch
pub fn chill() -> usize {
    // Switch to kernel frame
    frame::switch_to_kernel_frame();

    // Idle has to run in supervisor mode
    unsafe {
        sstatus::set_spp(sstatus::SPP::Supervisor);
    }

    just_chill_out_brah as usize
}

#[no_mangle]
extern "C" fn just_chill_out_brah() {
    #[allow(clippy::empty_loop)]
    loop {}
}
