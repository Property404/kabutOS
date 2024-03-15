use core::{
    fmt::{self, Display},
    result::Result,
};

/// Krabby-abi error type
#[derive(Debug)]
#[non_exhaustive]
pub enum KrabbyAbiError {
    InvalidPid(usize),
}

impl Display for KrabbyAbiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::InvalidPid(pid) => {
                write!(f, "Invalid PID: {pid}")
            }
        }
    }
}
