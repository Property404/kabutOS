use super::{FileRef, FileRefImpl, FileSystem};
use crate::{
    drivers::{BlockDriver, Driver},
    prelude::*,
};
use alloc::sync::Arc;
use core::cmp;
use fatfs::{FileSystem as Fat, IoBase, IoError, Read, Seek, SeekFrom, Write};
use spin::Mutex;

impl From<fatfs::Error<Self>> for KernelError {
    fn from(err: fatfs::Error<Self>) -> Self {
        match err {
            fatfs::Error::Io(err) => err,
            err => Self::FileSystem(Box::new(err)),
        }
    }
}

impl IoError for KernelError {
    fn is_interrupted(&self) -> bool {
        false
    }
    fn new_unexpected_eof_error() -> Self {
        todo!()
    }
    fn new_write_zero_error() -> Self {
        todo!()
    }
}

impl IoBase for Hal {
    type Error = KernelError;
}

struct Hal {
    capacity: usize,
    sector_size: usize,
    pos: usize,
    temp_buffer: [u8; 512],
    driver: Arc<Mutex<Driver<dyn BlockDriver>>>,
}

impl Hal {
    fn new(driver: Arc<Mutex<Driver<dyn BlockDriver>>>) -> KernelResult<Self> {
        let (capacity, sector_size) = {
            let mut driver = driver.lock();
            (driver.coupling.capacity()?, driver.coupling.sector_size()?)
        };
        Ok(Self {
            capacity,
            sector_size,
            pos: 0,
            temp_buffer: [0; 512],
            driver,
        })
    }
}

impl Seek for Hal {
    // Required method
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let pos = match pos {
            SeekFrom::Start(val) => val,
            SeekFrom::Current(val) => self
                .pos
                .checked_add_signed(isize::try_from(val)?)
                .ok_or(KernelError::Conversion)?
                .try_into()?,
            SeekFrom::End(val) => u64::try_from(self.capacity)?
                .checked_add_signed(val)
                .ok_or(KernelError::Conversion)?,
        };
        self.pos = pos.try_into()?;
        Ok(pos)
    }
}

impl Read for Hal {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        assert!(!buf.is_empty());

        let bytes_to_read = buf.len();
        let sector_size = self.sector_size;
        let mut start = self.pos;
        let end = self.pos + bytes_to_read;

        let mut driver = self.driver.lock();
        let driver = &mut driver.coupling;

        while start < end {
            let start_offset = start - start.align_down(sector_size);
            driver.read_blocking(start.align_down(sector_size), &mut self.temp_buffer)?;

            buf[..cmp::min(sector_size - start_offset, end - start)].copy_from_slice(
                &self.temp_buffer[start_offset
                    ..(cmp::min(sector_size - start_offset, end - start) + start_offset)],
            );

            start = start.align_next(sector_size);
        }

        self.pos += bytes_to_read;

        Ok(bytes_to_read)
    }
}

impl Write for Hal {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!("impl Write")
    }
}

#[derive(Debug)]
struct Fat32FileRefImpl;

impl FileRefImpl for Fat32FileRefImpl {
    fn read_blocking(&mut self, _buffer: &mut [u8]) -> KernelResult<()> {
        todo!()
    }
}

pub struct Fat32FileSystem {
    fat: Fat<Hal, fatfs::NullTimeProvider, fatfs::LossyOemCpConverter>,
}

impl FileSystem for Fat32FileSystem {
    fn open_blocking(&mut self, _path: impl AsRef<str>) -> KernelResult<FileRef> {
        Ok(FileRef(Box::new(Fat32FileRefImpl)))
    }

    fn list_blocking(&mut self, path: impl AsRef<str>) -> KernelResult<Vec<String>> {
        let dir = self.fat.root_dir().open_dir(path.as_ref()).unwrap();
        let mut vec = Vec::new();
        for item in dir.iter() {
            vec.push(item.unwrap().file_name());
        }
        Ok(vec)
    }

    fn read_blocking(
        &mut self,
        path: impl AsRef<str>,
        buffer: &mut [u8],
        offset: usize,
    ) -> KernelResult<()> {
        let mut file = self.fat.root_dir().open_file(path.as_ref())?;
        file.seek(SeekFrom::Start(offset.try_into()?))?;
        file.read_exact(buffer)?;
        Ok(())
    }
}

impl Fat32FileSystem {
    pub fn new(driver: Arc<Mutex<Driver<dyn BlockDriver>>>) -> KernelResult<Self> {
        Ok(Self {
            fat: Fat::new(Hal::new(driver)?, Default::default()).unwrap(),
        })
    }
}
