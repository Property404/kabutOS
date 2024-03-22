/// Syscall number
#[derive(Copy, Clone, enumn::N)]
#[repr(usize)]
pub enum Syscall {
    PutChar = 1,
    GetChar,
    PutString,
    Pinfo,
    Fork,
    Exit,
    WaitPid,
    Sleep,
    RequestMemory,
    PowerOff,
}
