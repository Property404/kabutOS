use crate::KrabbyAbiError;
use core::fmt::{self, Display};

/// File descriptor type
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDescriptor(u16);

impl From<FileDescriptor> for usize {
    fn from(fd: FileDescriptor) -> Self {
        fd.0.into()
    }
}

impl TryFrom<usize> for FileDescriptor {
    type Error = KrabbyAbiError;
    fn try_from(fd: usize) -> Result<Self, KrabbyAbiError> {
        Ok(Self(
            fd.try_into()
                .map_err(|_| KrabbyAbiError::InvalidFileDescriptor(fd))?,
        ))
    }
}

impl Display for FileDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}
