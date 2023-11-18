//! Error and Result type for use in this crate
use core::{fmt::Error as FmtError, result::Result, str::Utf8Error};
use embedded_line_edit::LineEditError;
use utf8_parser::Utf8ParserError;

/// Error type for use in the Kernel
#[derive(Debug)]
pub enum KernelError {
    /// Argument is invalid
    InvalidArguments,
    /// Driver is uninitialized
    DriverUninitialized,
    /// Converted from [core::fmt::Error]
    FmtError(FmtError),
    /// Converted from [core::str::Utf8Error]
    Utf8Error(Utf8Error),
    /// Converted from [utf8_parser::Utf8ParserError]
    Utf8ParserError(Utf8ParserError),

    /// Converted from [embedded_line_edit::LineEditError]
    LineEditError(LineEditError),
}

impl From<LineEditError> for KernelError {
    fn from(error: LineEditError) -> Self {
        KernelError::LineEditError(error)
    }
}

impl From<Utf8Error> for KernelError {
    fn from(error: Utf8Error) -> Self {
        KernelError::Utf8Error(error)
    }
}

impl From<Utf8ParserError> for KernelError {
    fn from(error: Utf8ParserError) -> Self {
        KernelError::Utf8ParserError(error)
    }
}

impl From<FmtError> for KernelError {
    fn from(error: FmtError) -> Self {
        KernelError::FmtError(error)
    }
}

/// Result type for use in this crate
pub type KernelResult<T = ()> = Result<T, KernelError>;
