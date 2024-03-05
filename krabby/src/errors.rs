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
    #[display("{}", _0)]
    Generic(&'static str),
    /// Attempted to dereference null pointer
    #[display("Attempted to dereference null pointer")]
    NullPointer,
    /// Driver is uninitialized
    #[display("Driver is uninitialized")]
    DriverUninitialized,
    /// Invalid address
    #[display("Invalid virtual address: {}", _0)]
    InvalidVirtualAddress(usize),
    /// Invalid address
    #[display("Invalid physical address: {}", _0)]
    InvalidPhysicalAddress(usize),
    /// Address is misaligned
    #[display("Address not page aligned: {}", _0)]
    AddressNotPageAligned(usize),
    /// Address is not mapped to kernel space
    #[display("Not mapped: {}", _0)]
    NotMapped(usize),
    /// Misaligned size
    #[display("Size is misaligned: {}", _0)]
    SizeMisaligned(usize),
    /// Missing FDT node property
    #[display("Missing FDT node property: {}", _0)]
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
