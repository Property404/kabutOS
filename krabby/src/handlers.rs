//! Rust IRQ and exception handlers
use riscv::register;

#[no_mangle]
fn exception_handler() {
    let mcause = register::mcause::read();
    let scause = register::scause::read();
    let mtval = register::mtval::read();
    let stval = register::stval::read();
    let mcause = (mcause.cause(), mcause.code());
    let scause = (scause.cause(), scause.code());
    panic!(
        "Exception:
    mcause: {mcause:?}
    mtval: 0x{mtval:08x}

    scause: {scause:?}
    stval: 0x{stval:08x}"
    );
}
