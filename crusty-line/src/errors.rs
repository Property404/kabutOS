use core::fmt::{self, Error as FmtError};
use embedded_line_edit::LineEditError;

/// Error type used in this crate
#[derive(Clone, Debug)]
pub enum CrustyLineError {
    /// Input ended unexpectedly
    UnexpectedEndOfInput,
    /// Error reading
    ReaderError,
    /// Converted from [core::fmt::Error]
    FmtError(FmtError),
    /// Converted from [embedded_line_edit::LineEditError]
    LineEditError(LineEditError),
}

impl From<FmtError> for CrustyLineError {
    fn from(other: FmtError) -> Self {
        Self::FmtError(other)
    }
}

impl From<LineEditError> for CrustyLineError {
    fn from(other: LineEditError) -> Self {
        Self::LineEditError(other)
    }
}

impl fmt::Display for CrustyLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEndOfInput => {
                write!(f, "Unexpected end of input")
            }
            Self::ReaderError => {
                write!(f, "Error reading input")
            }
            Self::FmtError(inner) => inner.fmt(f),
            Self::LineEditError(inner) => inner.fmt(f),
        }
    }
}

/// Result type used in this crate
pub type CrustyLineResult<T = ()> = Result<T, CrustyLineError>;
