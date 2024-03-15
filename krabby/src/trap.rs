//! Rust IRQ and exception handlers
use crate::{frame::TrapFrame, prelude::*, scheduler, syscalls::syscall_handler};
use owo_colors::OwoColorize;
use riscv::register::{
    self,
    scause::{Exception, Interrupt, Trap},
    sstatus::SPP,
};

#[no_mangle]
extern "C" fn exception_handler(
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
) {
    let trap_frame = register::sscratch::read() as *mut TrapFrame;
    let trap_frame = unsafe { trap_frame.as_mut().unwrap() };
    let scause = register::scause::read().cause();
    let mut pc = register::sepc::read();

    if matches!(scause, Trap::Exception(_)) {
        pc += 4;
    }

    // set PC
    if let Some(pid) = trap_frame.pid {
        let _ = scheduler::with_process(pid, |p| {
            p.pc = pc;
            Ok(())
        });
    }

    let rv = match scause {
        Trap::Exception(exception) => match exception {
            Exception::UserEnvCall => {
                let rv = syscall_handler(trap_frame, a7, [a0, a1, a2, a3, a4, a5, a6]);
                pc = scheduler::switch_processes(HartId::zero());
                rv
            }
            _ => unhandled_exception(trap_frame),
        },
        Trap::Interrupt(interrupt) => {
            match interrupt {
                Interrupt::SupervisorSoft => unsafe {
                    register::sip::clear_ssoft();
                    if register::sstatus::read().spp() == SPP::User {
                        pc = scheduler::switch_processes(HartId::zero());
                    }
                },
                _ => {
                    panic!("Unhandled interrupt!");
                }
            }
            Ok(())
        }
    };

    if let Err(err) = rv {
        println!("<trap error: {err}>");
    }

    register::sepc::write(pc);
}

fn unhandled_exception(trap_frame: &TrapFrame) -> ! {
    let scause = register::scause::read();
    let stval = register::stval::read();
    let scause = scause.cause();
    let pc = register::sepc::read();

    println!("{}", "UNHANDLED EXCEPTION".red());
    println!("Kernel frame: {:08x}", trap_frame.kernel_frame);
    println!("satp: {:08x}", trap_frame.satp);
    println!("sepc: {pc:08x}");

    panic!(
        "Unhandled Exception:
            scause: {scause:?}
            stval: 0x{stval:08x}"
    );
}
