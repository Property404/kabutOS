//! Utility functions

/// Align `val` up to nearest `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
#[must_use]
pub const fn align_up<const SIZE: usize>(val: usize) -> usize {
    let mut rv = align_down::<SIZE>(val);
    if !aligned::<SIZE>(val) {
        rv += SIZE;
    }
    rv
}

/// Align `val` up to next `SIZE`. Unlike [align_up], the value *always* increases
///
/// # Panics
/// If `SIZE` is not a power of two
#[must_use]
pub const fn align_next<const SIZE: usize>(val: usize) -> usize {
    align_down::<SIZE>(val) + SIZE
}

/// Align `val` down to nearest `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
#[must_use]
pub const fn align_down<const SIZE: usize>(val: usize) -> usize {
    assert!(SIZE.is_power_of_two());
    val & !(SIZE - 1)
}

/// Return true if `val` is aligned to `SIZE`
///
/// # Panics
/// If `SIZE` is not a power of two
#[must_use]
pub const fn aligned<const SIZE: usize>(val: usize) -> bool {
    assert!(SIZE.is_power_of_two());
    val & (SIZE - 1) == 0
}

/// Extra methods for `usize`
pub trait UsizeExt: Copy + Into<usize> {
    fn align_down(self, alignment: usize) -> usize {
        assert!(alignment.is_power_of_two());
        self.into() & !(alignment - 1)
    }

    fn align_next(self, alignment: usize) -> usize {
        assert!(alignment.is_power_of_two());
        self.align_down(alignment) + alignment
    }

    fn is_aligned_to(self, alignment: usize) -> bool {
        assert!(alignment.is_power_of_two());
        self.into() & (alignment - 1) == 0
    }

    fn from_u32(val: u32) -> usize {
        val.try_into().unwrap()
    }
}

impl UsizeExt for usize {}

#[cfg(feature = "test")]
pub fn test() {
    assert!(aligned::<1024>(0));
    assert!(aligned::<1024>(1024));
    assert!(aligned::<1024>(2048));
    assert!(!aligned::<1024>(1));
    assert!(!aligned::<1024>(1025));

    assert_eq!(align_down::<1024>(1024), 1024);
    assert_eq!(align_down::<1024>(5), 0);
    assert_eq!(align_down::<1024>(2047), 1024);
    assert_eq!(align_down::<1024>(2048), 2048);

    assert_eq!(align_up::<1024>(1024), 1024);
    assert_eq!(align_up::<1024>(5), 1024);
    assert_eq!(align_up::<1024>(2047), 2048);
    assert_eq!(align_up::<1024>(2048), 2048);

    assert_eq!(align_next::<1024>(1024), 2048);
    assert_eq!(align_next::<1024>(5), 1024);
    assert_eq!(align_next::<1024>(2047), 2048);
    assert_eq!(align_next::<1024>(2048), 1024 * 3);
}
