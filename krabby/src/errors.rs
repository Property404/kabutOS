//! Error and Result type for use in this crate
use core::{fmt::Error as FmtError, result::Result, str::Utf8Error};
use derive_more::{Display, From};
use embedded_line_edit::LineEditError;
use utf8_parser::Utf8ParserError;

/// Error type for use in the Kernel
#[derive(From, Debug, Display)]
pub enum KernelError {
    /// Argument is invalid
    #[display(fmt = "Argument is invalid")]
    InvalidArguments,
    /// Driver is uninitialized
    #[display(fmt = "Driver is uninitialized")]
    DriverUninitialized,
    /// Converted from [core::fmt::Error]
    #[from]
    FmtError(FmtError),
    /// Converted from [core::str::Utf8Error]
    #[from]
    Utf8Error(Utf8Error),
    /// Converted from [utf8_parser::Utf8ParserError]
    #[from]
    Utf8ParserError(Utf8ParserError),
    /// Converted from [embedded_line_edit::LineEditError]
    #[from]
    LineEditError(LineEditError),
}

/// Result type for use in this crate
pub type KernelResult<T = ()> = Result<T, KernelError>;
