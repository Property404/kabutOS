//! Error and Result type for use in this crate
use core::{fmt::Error as FmtError, num::ParseIntError, result::Result, str::Utf8Error};
use derive_more::{Display, From};
use embedded_line_edit::LineEditError;
use schmargs::{SchmargsError, StrippedSchmargsError};
use utf8_parser::Utf8ParserError;

/// Error type for use in the Kernel
#[derive(From, Debug, Display)]
pub enum KernelError {
    /// Generic error
    #[display(fmt = "{}", _0)]
    Generic(&'static str),
    /// Argument is invalid
    #[display(fmt = "Argument is invalid")]
    InvalidArguments,
    /// Driver is uninitialized
    #[display(fmt = "Driver is uninitialized")]
    DriverUninitialized,
    /// Invalid address
    #[display(fmt = "Invalid virtual address: {}", _0)]
    InvalidVirtualAddress(usize),
    /// Invalid address
    #[display(fmt = "Invalid physical address: {}", _0)]
    InvalidPhysicalAddress(usize),
    /// Address is misaligned
    #[display(fmt = "Address not page aligned: {}", _0)]
    AddressNotPageAligned(usize),
    /// Misaligned size
    #[display(fmt = "Size is misaligned: {}", _0)]
    SizeMisaligned(usize),
    /// Missing FDT node property
    #[display(fmt = "Missing FDT node property: {}", _0)]
    MissingProperty(&'static str),
    /// Converted from [core::fmt::Error]
    #[from]
    FmtError(FmtError),
    /// Converted from [core::str::Utf8Error]
    #[from]
    Utf8Error(Utf8Error),
    /// Converted from [core::num::ParseIntError]
    #[from]
    ParseIntError(ParseIntError),
    /// Converted from [utf8_parser::Utf8ParserError]
    #[from]
    Utf8ParserError(Utf8ParserError),
    /// Converted from [embedded_line_edit::LineEditError]
    #[from]
    LineEditError(LineEditError),
    /// Converted from [schmargs::SchmargsError] or [schmargs::StrippedSchmargsError]
    #[from]
    SchmargsError(StrippedSchmargsError),
}

impl<T> From<SchmargsError<T>> for KernelError {
    fn from(error: SchmargsError<T>) -> KernelError {
        error.strip().into()
    }
}

/// Result type for use in this crate
pub type KernelResult<T> = Result<T, KernelError>;
