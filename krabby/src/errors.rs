//! Error and Result type for use in this crate
use crate::prelude::*;
use core::{
    fmt::Error as FmtError,
    num::{ParseIntError, TryFromIntError},
    str::Utf8Error,
};
use derive_more::{Display, From};
use embedded_line_edit::LineEditError;
use krabby_abi::KrabbyAbiError;
use schmargs::{SchmargsError, StrippedSchmargsError};
use utf8_parser::Utf8ParserError;

/// Error type for use in the Kernel
#[derive(From, Debug, Display)]
pub enum KernelError {
    /// Generic error
    #[display("{}", _0)]
    Generic(&'static str),
    /// Invalid arguments to syscall
    #[display("Invalid arguments")]
    InvalidArguments,
    /// Unexpected end of input
    #[display("Unexpected end of input")]
    EndOfInput,
    /// Attempted to access forbidden page
    #[display("Forbidden page")]
    ForbiddenPage,
    /// No such syscall
    #[display("Invalid syscall: {}", _0)]
    InvalidSyscall(usize),
    /// No such process
    #[display("Process not found: {}", _0)]
    ProcessNotFound(Pid),
    /// Invalid PID
    #[display("Invalid PID: {}", _0)]
    InvalidPid(usize),
    /// Invalid Interrupt ID
    #[display("Invalid interruptId: {}", _0)]
    InvalidIntId(usize),
    /// Attempted to dereference null pointer
    #[display("Attempted to dereference null pointer")]
    NullPointer,
    /// Driver is uninitialized
    #[display("Driver is uninitialized")]
    DriverUninitialized,
    /// Interrupt unavailable
    #[display("Interrupt is unavailable")]
    InterruptUnavailable,
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
    /// Converted from [core::num::TryFromIntError]
    #[from]
    TryFromIntError(TryFromIntError),
    /// Converted from [krabby_abi::KrabbyAbiError]
    #[from]
    KrabbyAbiError(KrabbyAbiError),
}

impl From<KernelError> for usize {
    fn from(_: KernelError) -> Self {
        1
    }
}

impl<T> From<SchmargsError<T>> for KernelError {
    fn from(error: SchmargsError<T>) -> KernelError {
        error.strip().into()
    }
}

/// Result type for use in this crate
pub type KernelResult<T = ()> = Result<T, KernelError>;
