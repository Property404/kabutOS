use crate::sys;
use core::alloc::Layout;
use lock_api::RawMutex;
use talc::*;

#[global_allocator]
static ALLOCATOR: Talck<DummyMutex, UserClaimer> = Talc::new(UserClaimer(Span::empty())).lock();

struct UserClaimer(Span);

fn sbrk(size: usize) -> Result<usize, ()> {
    sys::request_memory(size).map_err(|_| ())
}

impl OomHandler for UserClaimer {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let base = sbrk(0)?;
        let new_breakline = sbrk(layout.size())?;
        let growth = new_breakline.checked_sub(base).unwrap();

        let old_span = talc.oom_handler.0;

        if old_span.is_empty() {
            unsafe {
                talc.oom_handler.0 = talc.claim(Span::from_base_size(base as *mut u8, growth))?;
            }
        } else {
            unsafe {
                talc.oom_handler.0 = talc.extend(old_span, old_span.extend(0, growth));
            }
        }

        Ok(())
    }
}

struct DummyMutex;

unsafe impl RawMutex for DummyMutex {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = DummyMutex;

    fn lock(&self) {}

    fn try_lock(&self) -> bool {
        true
    }

    unsafe fn unlock(&self) {}

    fn is_locked(&self) -> bool {
        false
    }
}
