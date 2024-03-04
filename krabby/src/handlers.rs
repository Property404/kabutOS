//! Rust IRQ and exception handlers
use crate::{frame::TrapFrame, println};
use riscv::register::{
    self,
    scause::{Exception, Trap},
};

#[no_mangle]
fn exception_handler() {
    let scause = register::scause::read();
    let stval = register::stval::read();
    let scause = (scause.cause(), scause.code());
    let trap_frame = register::sscratch::read() as *const TrapFrame;
    let trap_frame: TrapFrame = unsafe { trap_frame.as_ref().unwrap().clone() };
    let mut pc = register::sepc::read();

    println!("Kernel frame: {:08x}", trap_frame.kernel_frame);
    println!("satp: {:08x}", trap_frame.satp);
    println!("sepc: {pc:08x}");

    match scause.0 {
        Trap::Exception(Exception::UserEnvCall) => {
            println!("Syscall!");
            pc += 4;
        }
        _ => {
            panic!(
                "Exception:
            scause: {scause:?}
            stval: 0x{stval:08x}"
            );
        }
    };

    register::sepc::write(pc);
    assert_eq!(register::sepc::read(), pc);
}
