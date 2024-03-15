use crate::{
    frame::{self, TrapFrame},
    mmu,
};
use riscv::{asm::wfi, register::sstatus};

/// Switch to idle "process" and return its PC
// Maybe we should factor out this functionality along with Process::switch
pub fn chill() -> usize {
    // Switch to kernel frame
    let tframe = riscv::register::sscratch::read() as *const TrapFrame;
    frame::set_current_trap_frame(tframe);

    // Set page tables
    let satp = unsafe { tframe.as_ref().unwrap().satp.try_into().unwrap() };
    mmu::set_root_page_table(0, satp);

    // Idle has to run in supervisor mode
    unsafe {
        sstatus::set_spp(sstatus::SPP::Supervisor);
    }

    just_chill_out_brah as usize
}

#[no_mangle]
extern "C" fn just_chill_out_brah() {
    loop {
        wfi();
    }
}
