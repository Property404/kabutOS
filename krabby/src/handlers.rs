//! Rust IRQ and exception handlers
use crate::{frame::TrapFrame, println, syscalls::syscall_handler};
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
) -> usize {
    let trap_frame = register::sscratch::read() as *const TrapFrame;
    let trap_frame = unsafe { trap_frame.as_ref().unwrap() };
    let scause = register::scause::read().cause();
    let mut pc = register::sepc::read();

    let rv = match scause {
        Trap::Exception(exception) => {
            pc += 4;
            match exception {
                Exception::UserEnvCall => {
                    syscall_handler(trap_frame, a7, [a0, a1, a2, a3, a4, a5, a6])
                }
                _ => unhandled_exception(trap_frame),
            }
        }
        Trap::Interrupt(interrupt) => {
            match interrupt {
                Interrupt::SupervisorSoft => unsafe {
                    register::sip::clear_ssoft();
                },
                _ => {
                    panic!("Unhandled interrupt!");
                }
            }
            Ok(())
        }
    };

    // TODO: convert errors into errcodes
    let rv = if let Err(err) = rv {
        println!("<trap error: {err}>");
        1
    } else {
        0
    };

    register::sepc::write(pc);

    rv
}

fn unhandled_exception(trap_frame: &TrapFrame) -> ! {
    let scause = register::scause::read();
    let stval = register::stval::read();
    let scause = scause.cause();
    let pc = register::sepc::read();

    println!("Kernel frame: {:08x}", trap_frame.kernel_frame);
    println!("satp: {:08x}", trap_frame.satp);
    println!("sepc: {pc:08x}");

    panic!(
        "Unhandled Exception:
            scause: {scause:?}
            stval: 0x{stval:08x}"
    );
}
