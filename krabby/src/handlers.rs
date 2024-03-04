//! Rust IRQ and exception handlers
use crate::{frame::TrapFrame, println};
use riscv::register;

#[no_mangle]
fn exception_handler() {
    let scause = register::scause::read();
    let stval = register::stval::read();
    let scause = (scause.cause(), scause.code());
    let trap_frame = register::sscratch::read() as *const TrapFrame;
    let trap_frame: TrapFrame = unsafe { trap_frame.as_ref().unwrap().clone() };

    println!("Registers:");
    for i in 1..=31 {
        println!("\tx{i}: {:08x}", trap_frame.regs[i]);
    }

    panic!(
        "Exception:
    scause: {scause:?}
    stval: 0x{stval:08x}"
    );
}
