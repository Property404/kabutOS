use crate::{println, process::Process};

/// Main scheduler loop
pub fn run_scheduler() {
    println!("Running scheduler");
    let mut idle_process = Process::new(idle as *const (), 4096).unwrap();
    idle_process.run();
}

fn idle() {
    loop {
        riscv::asm::wfi();
    }
}
