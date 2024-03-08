use crate::{print, KernelError, KernelResult};

#[repr(usize)]
#[derive(Copy, Clone, enumn::N)]
enum Syscall {
    PutChar = 1,
}

/// Handle ecall exception
pub fn syscall_handler(call: usize, args: [usize; 7]) -> KernelResult<()> {
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
    };
    Ok(())
}
