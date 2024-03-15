use crate::KrabbyAbiError;
use core::{
    fmt::{self, Display},
    num::NonZeroU16,
    result::Result,
};

/// Process ID type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pid(NonZeroU16);

impl From<Pid> for u16 {
    fn from(pid: Pid) -> Self {
        u16::from(pid.0)
    }
}

impl From<Pid> for usize {
    fn from(pid: Pid) -> Self {
        u16::from(pid).into()
    }
}

impl TryFrom<usize> for Pid {
    type Error = KrabbyAbiError;
    fn try_from(pid: usize) -> Result<Self, KrabbyAbiError> {
        Self::maybe_from_usize(pid)?.ok_or(KrabbyAbiError::InvalidPid(pid))
    }
}

impl Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        NonZeroU16::fmt(&self.0, f)
    }
}

impl Pid {
    /// Return None if zero, else return a Pid
    pub fn maybe_from_u16(val: u16) -> Option<Self> {
        NonZeroU16::new(val).map(Self)
    }

    /// Return None if zero, else return a Pid
    pub fn maybe_from_usize(val: usize) -> Result<Option<Self>, KrabbyAbiError> {
        let val: u16 = val
            .try_into()
            .map_err(|_| KrabbyAbiError::InvalidPid(val))?;
        Ok(Self::maybe_from_u16(val))
    }
}
