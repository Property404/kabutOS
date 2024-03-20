use crate::{
    idle,
    prelude::*,
    process::{BlockCondition, Process, ProcessState},
    timer::Instant,
    KernelError, KernelResult,
};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use riscv::register::{sepc, sstatus};
use spin::Mutex;

// Only supporting one hart currently
const MAX_HARTS: usize = 1;

// Processes lists are per CPU core
static PROCESSES: Mutex<Vec<Process>> = Mutex::new(Vec::new());

extern "C" {
    fn enter_user_mode();
}

/// Add a process to the scheduler
pub fn add_process(process: Process) {
    PROCESSES.lock().push(process);
}

/// Start the scheduler
pub fn start_with(process: Process) {
    add_process(process);
    let pc = switch_processes(HartId::zero());

    unsafe {
        sstatus::set_spp(sstatus::SPP::User);
        sepc::write(pc);
        enter_user_mode();
    }
}

/// Change up processes
///
/// Returns the new program counter
pub fn switch_processes(hart_id: HartId) -> usize {
    schedule_inner(hart_id, &mut PROCESSES.lock())
}

/// Run method over process `pid`
pub fn with_process<T>(pid: Pid, f: impl Fn(&mut Process) -> KernelResult<T>) -> KernelResult<T> {
    let mut processes = PROCESSES.lock();
    for proc in processes.iter_mut() {
        if proc.pid == pid {
            return f(proc);
        }
    }
    Err(KernelError::ProcessNotFound(pid))
}

fn reap(processes: &mut Vec<Process>) {
    let mut zombies = Vec::new();

    let mut i = 0;
    let mut len = processes.len();
    while i < len {
        match processes[i].state {
            ProcessState::Zombie(_) => {
                zombies.push(processes.swap_remove(i));
                len -= 1;
            }
            ProcessState::Blocked(BlockCondition::Until(instant)) => {
                let now = Instant::now();
                if now >= instant {
                    processes[i].unblock();
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Unblock processes waiting on deaths
    for zombie in zombies {
        let ProcessState::Zombie(res) = zombie.state else {
            panic!("Found non-zombie in zombie list!");
        };

        for process in processes.iter_mut() {
            let ProcessState::Blocked(condition) = process.state else {
                continue;
            };
            let BlockCondition::OnDeathOfPid(blocked_on_pid) = condition else {
                continue;
            };

            if zombie.pid == blocked_on_pid {
                process.frame.as_mut().set_exit_value(res);
                process.unblock();
            }
        }
    }
}

// Round-robin scheduler
fn schedule_inner(hart_id: HartId, processes: &mut Vec<Process>) -> usize {
    assert!(usize::from(hart_id) < MAX_HARTS);
    assert!(!processes.is_empty());

    // Reap any zombie processes
    reap(processes);

    // Pause all processes
    for process in processes.iter_mut() {
        if process.state == ProcessState::Running {
            process.pause();
            break;
        }
    }

    // TODO(optimization): pick a proper ordering
    // SeqCst is the safest
    static INDEX: AtomicUsize = AtomicUsize::new(1);
    let mut index = INDEX.fetch_add(1, Ordering::SeqCst);
    let len = processes.len();

    // Run the next non-blocked process
    for _ in 0..len {
        let process: &mut Process = processes.get_mut(index % len).expect("out-of-bounds");
        if process.is_blocked() {
            index += 1;
            continue;
        }
        process.switch();
        return process.pc;
    }

    idle::chill()
}
