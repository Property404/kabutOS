//! Rust IRQ and exception handlers
use riscv::register;

#[no_mangle]
fn exception_handler() {
    let scause = register::scause::read();
    let stval = register::stval::read();
    let scause = (scause.cause(), scause.code());
    panic!(
        "Exception:
    scause: {scause:?}
    stval: 0x{stval:08x}"
    );
}
