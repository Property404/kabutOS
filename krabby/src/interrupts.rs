use crate::{drivers::DRIVERS, prelude::*};
use alloc::collections::BTreeMap;
use spin::RwLock;

static INTERRUPT_HANDLERS: RwLock<BTreeMap<InterruptId, InterruptHandler>> =
    RwLock::new(BTreeMap::new());
pub struct InterruptHandler(Box<dyn Fn(InterruptId) -> KernelResult + Send + Sync>);

/// Register interrupt handler
pub fn register_handler(
    id: InterruptId,
    func: impl Fn(InterruptId) -> KernelResult<()> + Send + Sync + 'static,
) {
    let mut handlers = INTERRUPT_HANDLERS.write();

    if handlers.contains_key(&id) {
        panic!("Interrupt handler already registered!");
    }

    handlers.insert(id, InterruptHandler(Box::new(func)));
}

/// Run next interrupt handler
pub fn run_next_handler() -> KernelResult {
    let mut driver = DRIVERS.ic.lock();
    if let Some(driver) = &mut *driver {
        let claim = driver.next().expect("Expected claim");
        run_handler(claim)
    } else {
        warn!("No interrupt controller");
        Ok(())
    }
}

// Run interrupt handler
pub fn run_handler(id: InterruptId) -> KernelResult {
    let handlers = INTERRUPT_HANDLERS.read();
    if let Some(handler) = handlers.get(&id) {
        handler.0(id)
    } else {
        Err(KernelError::InterruptUnavailable)
    }
}
