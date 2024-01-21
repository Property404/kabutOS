//! Rust IRQ and exception handlers
use riscv::register::mcause;

#[no_mangle]
fn exception_handler() {
    let cause = mcause::read().cause();
    panic!("Exception: {cause:?}");
}
