use crate::{
    prelude::*,
    process::{Process, ProcessState},
};
use alloc::vec::Vec;
use core::{
    cell::RefCell,
    sync::atomic::{AtomicUsize, Ordering},
};
use critical_section::Mutex;
use riscv::register::sstatus;

// Only supporting one hart currently
const MAX_HARTS: usize = 1;

// Processes lists are per CPU core
static PROCESSES: Mutex<RefCell<Vec<Process>>> = Mutex::new(RefCell::new(Vec::new()));

extern "C" {
    fn run_process(addr: usize);
}

/// Add a process to the scheduler
pub fn add_process(process: Process) {
    critical_section::with(|cs| PROCESSES.borrow_ref_mut(cs).push(process));
}

/// Start the scheduler
pub fn start_with(process: Process) {
    add_process(process);
    let pc = switch_processes(HartId::zero(), 0xDEADBEEF);

    unsafe {
        sstatus::set_spp(sstatus::SPP::User);
        run_process(pc);
    }
}

/// Change up processes
///
/// Returns the new program counter
pub fn switch_processes(hart_id: HartId, pc: usize) -> usize {
    critical_section::with(|cs| schedule_inner(hart_id, pc, &mut PROCESSES.borrow_ref_mut(cs)))
}

// Round-robin scheduler
fn schedule_inner(hart_id: HartId, pc: usize, processes: &mut [Process]) -> usize {
    assert!(usize::from(hart_id) < MAX_HARTS);
    assert!(!processes.is_empty());

    // TODO(optimization): pick a proper ordering
    // SeqCst is the safest
    static INDEX: AtomicUsize = AtomicUsize::new(1);
    let index = INDEX.fetch_add(1, Ordering::SeqCst);

    for process in processes.iter_mut() {
        if process.state == ProcessState::RUNNING {
            process.pc = pc;
            process.pause();
            break;
        }
    }

    let process: &mut Process = processes
        .get_mut(index % processes.len())
        .expect("out-of-bounds");
    process.switch();
    process.pc
}
