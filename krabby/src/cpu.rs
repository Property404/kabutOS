use core::{
    arch::asm,
    fmt::{self, Display},
    result::Result,
};
use derive_more::{From, Into};

/// Hart (hardware thread) ID type
#[derive(Copy, From, Into, Clone, PartialEq, Eq, Debug)]
#[into(usize)]
#[from(u16)]
#[repr(transparent)]
pub struct HartId(usize); // Maybe this can shrink?

impl HartId {
    /// Hart 0 (primary processor)
    pub const fn zero() -> Self {
        Self(0)
    }

    /// If this is hart 0 (primary processor)
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Display for HartId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

/// Interrupt ID type
#[derive(Copy, From, Into, Clone, PartialEq, Eq, Debug)]
pub struct InterruptId(u32);

impl Display for InterruptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl From<InterruptId> for usize {
    fn from(other: InterruptId) -> Self {
        other.0.try_into().expect("Expect usize to hold u32")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// A RISC-V register (RVE not supported)
pub enum Register {
    ReturnAddress = 1,
    StackPointer,
    GlobalPointer,
    /// Argment 0 or return value
    Arg0 = 10,
    /// Argument 1 or secondary return value
    Arg1,
    Arg2,
    Arg3,
    Arg4,
    Arg5,
    Arg6,
    Arg7,
}

impl Register {
    /// Get current value of this register
    pub fn value(self) -> usize {
        let mut val: usize;
        use Register::*;
        unsafe {
            match self {
                ReturnAddress => asm!("mv {}, ra", out(reg) val),
                StackPointer => asm!("mv {}, sp", out(reg) val),
                GlobalPointer => asm!("mv {}, gp", out(reg) val),
                Arg0 => asm!("mv {}, a0", out(reg) val),
                Arg1 => asm!("mv {}, a1", out(reg) val),
                Arg2 => asm!("mv {}, a2", out(reg) val),
                Arg3 => asm!("mv {}, a3", out(reg) val),
                Arg4 => asm!("mv {}, a4", out(reg) val),
                Arg5 => asm!("mv {}, a5", out(reg) val),
                Arg6 => asm!("mv {}, a6", out(reg) val),
                Arg7 => asm!("mv {}, a7", out(reg) val),
            }
        };
        val
    }

    /// Return as string slice, as would be seen in assembly code
    pub const fn as_str(&self) -> &'static str {
        use Register::*;
        match self {
            ReturnAddress => "ra",
            StackPointer => "sp",
            GlobalPointer => "gp",
            Arg0 => "a0",
            Arg1 => "a1",
            Arg2 => "a2",
            Arg3 => "a3",
            Arg4 => "a4",
            Arg5 => "a5",
            Arg6 => "a6",
            Arg7 => "a7",
        }
    }
}

impl From<Register> for usize {
    fn from(val: Register) -> Self {
        val as Self
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}
