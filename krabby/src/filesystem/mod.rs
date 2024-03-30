use crate::prelude::*;

pub mod fat32;

pub trait FileSystem {
    fn list_blocking(&mut self, path: impl AsRef<str>) -> KernelResult<Vec<String>>;
    fn read_blocking(
        &mut self,
        path: impl AsRef<str>,
        buffer: &mut [u8],
        offset: usize,
    ) -> KernelResult<()>;
}
