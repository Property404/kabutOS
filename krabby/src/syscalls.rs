use crate::{
    frame::TrapFrame,
    mmu::{self, PAGE_SIZE},
    prelude::*,
    scheduler,
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
    Pinfo = 3,
    Fork = 4,
    Exit = 5,
}

/// Handle ecall exception
pub fn syscall_handler(frame: &mut TrapFrame, call: usize, args: [usize; 7]) -> KernelResult<()> {
    let rv = syscall_inner(frame, call, args);
    frame.set_return_value(&rv);
    rv.map(|_| ())
}

fn syscall_inner(
    frame: &mut TrapFrame,
    call: usize,
    args: [usize; 7],
) -> KernelResult<SyscallResult> {
    let call = Syscall::n(call).ok_or(KernelError::InvalidSyscall(call))?;
    let pid = frame.pid.expect("Process without PID!");

    let rv = match call {
        Syscall::PutChar => {
            let ch = char::from_u32(
                args[0]
                    .try_into()
                    .map_err(|_| KernelError::InvalidArguments)?,
            )
            .ok_or(KernelError::InvalidArguments)?;
            print!("{ch}");
            SyscallResult::Success
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
            SyscallResult::Success
        }
        Syscall::Pinfo => {
            // Currently just returning the PID, but later we can return all sorts of things
            SyscallResult::Value(pid.into())
        }
        Syscall::Fork => {
            let mut child = scheduler::with_process(pid, |p| p.fork())?;
            child
                .frame
                .as_mut()
                .set_return_value(&Ok(SyscallResult::Value(0)));
            child.pc += 4;
            let child_pid = child.pid;
            scheduler::add_process(child);
            SyscallResult::Value(child_pid.into())
        }
        Syscall::Exit => {
            scheduler::with_process(pid, |p| p.exit())?;
            SyscallResult::Success
        }
    };
    Ok(rv)
}

#[derive(Copy, Clone, Debug)]
enum SyscallResult {
    Success,
    Value(usize),
}

impl From<SyscallResult> for usize {
    fn from(res: SyscallResult) -> Self {
        match res {
            SyscallResult::Success => 0,
            SyscallResult::Value(val) => val,
        }
    }
}
