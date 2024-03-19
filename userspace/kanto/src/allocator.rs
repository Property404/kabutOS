use crate::sys;
use core::alloc::Layout;
use talc::*;

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, UserClaimer> = Talc::new(UserClaimer).lock();

struct UserClaimer;

impl OomHandler for UserClaimer {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let current_breakline = sys::request_memory(0).map_err(|_| ())?;
        let new_breakline = sys::request_memory(layout.size()).map_err(|_| ())?;
        assert!(new_breakline > current_breakline);
        let size = new_breakline.checked_sub(current_breakline).unwrap();

        unsafe {
            talc.claim(Span::from_base_size(current_breakline as *mut u8, size))
                .map(|_| ())
        }
    }
}
