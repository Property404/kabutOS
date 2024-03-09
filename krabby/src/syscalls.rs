use crate::{
    frame::TrapFrame,
    mmu::{self, PAGE_SIZE},
    prelude::*,
    util::*,
    KernelError, KernelResult,
};
use core::cmp;
use utf8_parser::Utf8Parser;

#[repr(usize)]
#[derive(Copy, Clone, enumn::N)]
enum Syscall {
    PutChar = 1,
    PutString = 2,
}

/// Handle ecall exception
pub fn syscall_handler(frame: &TrapFrame, call: usize, args: [usize; 7]) -> KernelResult<()> {
    let call = Syscall::n(call).ok_or(KernelError::InvalidSyscall(call))?;
    match call {
        Syscall::PutChar => {
            let ch = char::from_u32(
                args[0]
                    .try_into()
                    .map_err(|_| KernelError::InvalidArguments)?,
            )
            .ok_or(KernelError::InvalidArguments)?;
            print!("{ch}");
        }
        Syscall::PutString => {
            let table = frame.root_page_table();
            let mut parser = Utf8Parser::new();

            // This has to be done per page because the pages not may be contiguous in kernel space
            let mut start = args[0];
            let mut bytes_left = args[1];
            while bytes_left > 0 {
                {
                    let size = cmp::min(bytes_left, align_next::<PAGE_SIZE>(start) - start);
                    let start = usize::from(mmu::get_user_page(table, start.try_into()?)?);
                    let slice: &[u8] =
                        unsafe { core::slice::from_raw_parts(start as *const u8, size) };
                    for byte in slice {
                        if let Some(ch) = parser.push(*byte)? {
                            print!("{ch}");
                        }
                    }
                }

                bytes_left = bytes_left.saturating_sub(PAGE_SIZE);
                start += PAGE_SIZE;
            }
        }
    };
    Ok(())
}
