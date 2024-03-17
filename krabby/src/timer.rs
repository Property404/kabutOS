use crate::{drivers::DRIVERS, prelude::*};
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use spin::RwLock;

static INSTANT_IN_NS: AtomicU64 = AtomicU64::new(0);
static DURATION: RwLock<Duration> = RwLock::new(Duration::new(0, 0));

/// Some opaque monotonic time point - Similar to `Instant` in `std`
// Keeps track in nanos
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(u64);

impl Instant {
    /// Get the current instant
    pub fn now() -> Self {
        Instant(INSTANT_IN_NS.load(Ordering::Relaxed))
    }
}

/// Set the timer period
pub fn set_timer_period(hart: HartId, duration: Duration) -> KernelResult<()> {
    // Opening this lock should prevent race condition on multiple calls to set_timer() We don't
    // want duration_cache and the actual duration in the timer driver to get out-of-sync
    let mut duration_cache = DURATION.write();

    let timer = DRIVERS.timer.lock();
    let mut timer = timer.borrow_mut();
    if let Some(timer) = &mut *timer {
        timer.set_alarm(hart, duration);
        *duration_cache = duration;
        Ok(())
    } else {
        Err(KernelError::DriverUninitialized)
    }
}

/// Increment the current Instant
pub fn tick() -> KernelResult<()> {
    let duration = DURATION.read();
    let duration = duration.as_nanos().try_into()?;
    INSTANT_IN_NS.store(duration, Ordering::Relaxed);
    Ok(())
}
