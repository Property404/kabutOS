//! Rust IRQ and exception handlers
use crate::{
    frame::{self, TrapFrame},
    interrupts,
    mmu::PAGE_SIZE,
    prelude::*,
    scheduler,
    syscalls::syscall_handler,
    timer,
};
use core::{ffi::c_void, ptr};
use owo_colors::OwoColorize;
use riscv::register::{
    self,
    scause::{Exception, Interrupt, Trap},
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
    let trap_frame = frame::get_current_trap_frame_mut();
    let trap_frame = unsafe { trap_frame.as_mut().unwrap() };
    let scause = register::scause::read();
    let mut pc = register::sepc::read();

    frame::switch_to_kernel_frame();

    check_for_stack_overflow();

    // Exceptions are synchronous so the PC needs to move up
    if scause.is_exception() {
        pc += 4;
    }

    // set PC
    if let Some(pid) = trap_frame.pid {
        let _ = scheduler::with_process(pid, |p| {
            p.pc = pc;
            Ok(())
        });
    }

    let rv = match scause.cause() {
        Trap::Exception(exception) => match exception {
            Exception::UserEnvCall => {
                let rv = syscall_handler(trap_frame, a7, (a0, a1, a2, a3, a4, a5, a6));
                pc = scheduler::switch_processes(HartId::zero());
                rv
            }
            _ => unhandled_exception(trap_frame),
        },
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::SupervisorSoft => {
                if timer::tick().is_err() {
                    println!("[kernel: timer tick failed]");
                };
                unsafe {
                    register::sip::clear_ssoft();
                }
                pc = scheduler::switch_processes(HartId::zero());
                Ok(())
            }
            Interrupt::SupervisorExternal => {
                let res = interrupts::run_next_handler();
                pc = scheduler::switch_processes(HartId::zero());
                res
            }
            _ => {
                panic!("Unhandled interrupt!");
            }
        },
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
    println!("Kernel frame: {:p}", trap_frame.kernel_frame);
    println!("satp: {:08x}", trap_frame.satp);
    println!("sepc: {pc:08x}");

    panic!(
        "Unhandled Exception:
            scause: {scause:?}
            stval: 0x{stval:08x}"
    );
}

fn check_for_stack_overflow() {
    let stval = register::stval::read();
    let guard = unsafe { ptr::from_ref(&stack_guard) as usize };

    if stval >= guard && stval < guard + PAGE_SIZE {
        println!("[STACK_OVERFLOW]");
        panic!("Stack overflow");
    }
}

extern "C" {
    static stack_guard: c_void;
}
