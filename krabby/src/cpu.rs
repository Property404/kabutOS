use derive_more::{From, Into};

/// Hart (hardware thread) ID type
#[derive(Copy, From, Into, Clone, PartialEq, Eq, Debug)]
#[into(usize)]
#[from(u16)]
#[repr(transparent)]
pub struct HartId(usize); // Maybe this can shrink?

impl HartId {
    /// Hart 0 (primary processor)
    pub const fn zero() -> Self {
        Self(0)
    }

    /// If this is hart 0 (primary processor)
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}
