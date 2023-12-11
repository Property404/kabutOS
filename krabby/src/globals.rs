//! Global data that is initialized once and only once
use fdt::Fdt;

// WARNING: This needs to only be initialized ONCE
// by the main thread before any synchronization shit happens

static mut GLOBAL_DATA: Option<GlobalData> = None;

/// Global data structure
pub struct GlobalData {
    /// The device tree passed from the bootloader
    pub device_tree: Fdt<'static>,
}

/// Initialize global data
///
/// # Safety
///
/// This must be exactly once, and there must be no other reads during initialization. This means
/// not calling [get]
pub unsafe fn initialize(device_tree: Fdt<'static>) {
    unsafe {
        if GLOBAL_DATA.is_some() {
            panic!("Already initialized global data");
        }
        GLOBAL_DATA = Some(GlobalData { device_tree })
    }
}

/// Get global data
pub fn get() -> &'static GlobalData {
    unsafe { GLOBAL_DATA.as_ref().expect("Global data not initialized") }
}
