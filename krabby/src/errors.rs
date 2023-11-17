//! Error and Result type for use in this crate
use core::result::Result;
use core::str::Utf8Error;

/// Error type for use in the Kernel
#[derive(Debug)]
pub enum KernelError {
    /// Argument is invalid
    InvalidArguments = 1,
    /// Driver is uninitialized
    DriverUninitialized,
    /// Converted from [core::fmt::Error]
    FmtError = 100,
    /// Converted from [core::str::Utf8Error]
    Utf8Error,
}

impl From<Utf8Error> for KernelError {
    fn from(_error: Utf8Error) -> Self {
        KernelError::Utf8Error
    }
}

impl From<core::fmt::Error> for KernelError {
    fn from(_error: core::fmt::Error) -> Self {
        KernelError::FmtError
    }
}

/// Result type for use in this crate
pub type KernelResult<T = ()> = Result<T, KernelError>;
