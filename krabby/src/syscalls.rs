use crate::{
    drivers::DRIVERS,
    frame::TrapFrame,
    mmu::{self, PAGE_SIZE},
    prelude::*,
    process::BlockCondition,
    scheduler,
    timer::Instant,
    util::*,
};
use core::{cmp, str, time::Duration};
use krabby_abi::{ProcessError, Syscall};
use utf8_parser::Utf8Parser;

type Args = (usize, usize, usize, usize, usize, usize, usize);

/// Handle ecall exception
pub fn syscall_handler(frame: &mut TrapFrame, call: usize, args: Args) -> KernelResult<()> {
    let rv = syscall_inner(frame, call, args);
    frame.set_return_value(&rv);
    rv.map(|_| ())
}

fn syscall_inner(frame: &mut TrapFrame, call: usize, args: Args) -> KernelResult<SyscallResult> {
    let call = Syscall::n(call).ok_or(KernelError::InvalidSyscall(call))?;
    let pid = frame.pid.expect("Process without PID!");

    let rv = match call {
        Syscall::PutChar => {
            let ch = char::from_u32(
                args.0
                    .try_into()
                    .map_err(|_| KernelError::InvalidArguments)?,
            )
            .ok_or(KernelError::InvalidArguments)?;
            print!("{ch}");
            SyscallResult::Success
        }
        Syscall::GetChar => {
            if let Some(ch) = scheduler::with_process(pid, |p| {
                if let Some(c) = p.stdin_buffer.pop_front() {
                    Ok(Some(c))
                } else {
                    let uart = DRIVERS.uart.read();
                    let int_id: InterruptId = if let Some(uart) = &*uart {
                        let uart = uart.lock();
                        *uart
                            .info
                            .interrupts
                            .first()
                            .ok_or(KernelError::InterruptUnavailable)?
                    } else {
                        return Err(KernelError::DriverUninitialized);
                    };
                    p.block(BlockCondition::OnUart(int_id));
                    Ok(None)
                }
            })? {
                SyscallResult::Value(ch as usize)
            } else {
                SyscallResult::Success
            }
        }
        Syscall::PutString => {
            let table = frame.root_page_table();
            let mut parser = Utf8Parser::new();

            // This has to be done per page because the pages not may be contiguous in kernel space
            let mut start = args.0;
            let mut bytes_left = args.1;
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
            let child_pid = child.pid;
            scheduler::add_process(child);
            SyscallResult::Value(child_pid.into())
        }
        Syscall::Exit => {
            let res = ProcessError::n(args.1).map(Err).unwrap_or(Ok(()));
            scheduler::with_process(pid, |p| p.exit(res))?;
            SyscallResult::Success
        }
        Syscall::WaitPid => {
            let target_pid = Pid::try_from(args.0)?;

            // Return immediately if process is stopped already
            if scheduler::with_process(target_pid, |_| Ok(())).is_err() {
                return Ok(SyscallResult::Success);
            }

            scheduler::with_process(pid, |p| {
                p.block(BlockCondition::OnDeathOfPid(target_pid));
                Ok(())
            })?;
            SyscallResult::Success
        }
        Syscall::Sleep => {
            let duration = Duration::new(args.0.try_into()?, args.1.try_into()?);
            scheduler::with_process(pid, |p| {
                p.block(BlockCondition::Until(Instant::now() + duration));
                Ok(())
            })?;
            SyscallResult::Success
        }
        Syscall::RequestMemory => {
            let num_bytes = args.0;
            let res = scheduler::with_process(pid, |p| p.request_memory(num_bytes))?;
            SyscallResult::Value(res)
        }
        Syscall::PowerOff => {
            #[cfg(feature = "test")]
            crate::test::quit_qemu()?;
            unimplemented!();
        }
        // Development test aid
        // This does whatever I want it to do
        Syscall::Test => {
            if let Some(device) = &*DRIVERS.block.read() {
                let mut device = device.lock();
                let mut buffer = Box::new([0u8; 512]);

                println!("Starting read");
                device.coupling.start_read(0, buffer.as_mut())?;
                println!("Buffer: {}", str::from_utf8(&*buffer).unwrap());
            }

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
