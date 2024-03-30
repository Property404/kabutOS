use crate::prelude::*;
use core::fmt;

pub mod fat32;

/// Reference to an open file, meant to be stored in a process's file descriptor table
#[derive(Debug)]
pub struct FileRef(Box<dyn FileRefImpl>);

trait FileRefImpl: fmt::Debug + Send {
    fn read_blocking(&mut self, buffer: &mut [u8]) -> KernelResult<()>;
}

impl FileRefImpl for FileRef {
    fn read_blocking(&mut self, buffer: &mut [u8]) -> KernelResult<()> {
        self.0.read_blocking(buffer)
    }
}

pub trait FileSystem {
    fn open_blocking(&mut self, path: impl AsRef<str>) -> KernelResult<FileRef>;
    fn list_blocking(&mut self, path: impl AsRef<str>) -> KernelResult<Vec<String>>;
    fn read_blocking(
        &mut self,
        path: impl AsRef<str>,
        buffer: &mut [u8],
        offset: usize,
    ) -> KernelResult<()>;
}
