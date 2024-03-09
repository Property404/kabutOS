//! Utility functions

/// Align `val` up to nearest `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
pub const fn align_up<const SIZE: usize>(val: usize) -> usize {
    let mut rv = align_down::<SIZE>(val);
    if val % SIZE != 0 {
        rv += SIZE;
    }
    rv
}

/// Align `val` up to next `SIZE`. Unlike [align_up], the value *always* increases
///
/// # Panics
/// If `SIZE` is not a power of two
pub const fn align_next<const SIZE: usize>(val: usize) -> usize {
    align_down::<SIZE>(val) + SIZE
}

/// Align `val` down to nearest `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
pub const fn align_down<const SIZE: usize>(val: usize) -> usize {
    assert!(SIZE.is_power_of_two());
    SIZE * (val / SIZE)
}

/// Return true if `val` is aligned to `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
pub const fn aligned<const SIZE: usize>(val: usize) -> bool {
    assert!(SIZE.is_power_of_two());
    val % SIZE == 0
}
