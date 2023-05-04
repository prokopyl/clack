use clap_sys::process::*;
use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

/// Status returned by a plugin after processing.
///
/// This is mainly used as a way for the plugin to tell the host when it can be safely put to sleep.
///
/// Note that Clack uses a [`Result`] enum for relaying a failed processing to the host,
/// unlike the C CLAP API which uses an extra state in enum (`CLAP_PROCESS_ERROR`) to indicate failure.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessStatus {
    /// Processing should continue: the plugin has no desire to be put to sleep.
    Continue = CLAP_PROCESS_CONTINUE,
    /// Processing should continue, unless all outputs are quiet.
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    /// The plugin is currently processing its tail (e.g. release, reverb, etc.).
    ///
    /// Use the `tail` extension to query the plugin for its current tail length.
    Tail = CLAP_PROCESS_TAIL,
    /// No more processing is required until the next event or variation in audio input.
    Sleep = CLAP_PROCESS_SLEEP,
}

impl ProcessStatus {
    /// Gets a [`ProcessStatus`] from the raw, C-FFI compatible value.
    ///
    /// In order to match Clack's APIs, this returns `Some(Err(()))` if the value is
    /// `CLAP_PROCESS_ERROR`.
    ///
    /// If the given integer does not match any known CLAP Processing status codes, [`None`] is
    /// returned.
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
