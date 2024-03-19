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

/// Exit error code from a process
#[derive(enumn::N, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum ProcessError {
    /// Generic failure
    Failure = 1,
}

impl From<ProcessError> for usize {
    fn from(err: ProcessError) -> Self {
        err as usize
    }
}

pub type ProcessResult = Result<(), ProcessError>;
