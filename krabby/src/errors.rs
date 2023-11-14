//! Error and Result type for use in this crate
use core::result::Result;

/// Error type for use in the Kernel
#[derive(Debug)]
pub enum KernelError {
    /// Argument is invalid
    InvalidArguments = 1,
    /// Driver is uninitialized
    DriverUninitialized,
    /// Converted from [core::fmt::Error]
    FmtError,
}

impl From<core::fmt::Error> for KernelError {
    fn from(_error: core::fmt::Error) -> Self {
        KernelError::FmtError
    }
}

/// Result type for use in this crate
pub type KernelResult<T = ()> = Result<T, KernelError>;
