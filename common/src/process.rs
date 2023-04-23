use clap_sys::process::*;
use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    Sleep = CLAP_PROCESS_SLEEP,
    Tail = CLAP_PROCESS_TAIL,
}

impl ProcessStatus {
    pub fn from_raw(raw: clap_process_status) -> Option<Result<Self, ()>> {
        use ProcessStatus::*;

        match raw {
            CLAP_PROCESS_CONTINUE => Some(Ok(Continue)),
            CLAP_PROCESS_CONTINUE_IF_NOT_QUIET => Some(Ok(ContinueIfNotQuiet)),
            CLAP_PROCESS_SLEEP => Some(Ok(Sleep)),
            CLAP_PROCESS_TAIL => Some(Ok(Tail)),
            CLAP_PROCESS_ERROR => Some(Err(())),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ConstantMask(u64);

impl ConstantMask {
    pub const CAPACITY: u8 = 64;

    pub const FULLY_CONSTANT: ConstantMask = ConstantMask(u64::MAX);
    pub const FULLY_DYNAMIC: ConstantMask = ConstantMask(0);

    #[inline]
    pub const fn to_bits(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        ConstantMask(bits)
    }

    #[inline]
    pub const fn is_channel_constant(&self, channel_index: u64) -> bool {
        (self.0 & (1 << channel_index)) == 1
    }

    #[inline]
    pub fn set_channel_constant(&mut self, channel_index: u64, value: bool) {
        if value {
            self.0 |= 1 << channel_index
        } else {
            self.0 &= !(1 << channel_index)
        }
    }
}

impl Debug for ConstantMask {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        core::fmt::Binary::fmt(&self.0, f)
    }
}

impl BitAnd for ConstantMask {
    type Output = ConstantMask;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        ConstantMask(self.0 & rhs.0)
    }
}

impl BitOr for ConstantMask {
    type Output = ConstantMask;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        ConstantMask(self.0 | rhs.0)
    }
}

impl BitXor for ConstantMask {
    type Output = ConstantMask;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        ConstantMask(self.0 ^ rhs.0)
    }
}

impl BitAndAssign for ConstantMask {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for ConstantMask {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitXorAssign for ConstantMask {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
