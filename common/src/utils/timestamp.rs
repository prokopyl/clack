use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU64;
use std::time::Duration;

/// A timestamp, defined as the number of seconds since UNIX EPOCH.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Timestamp(NonZeroU64);

impl Timestamp {
    /// Returns the timestamp from its raw, C-FFI compatible representation.
    ///
    /// This returns `None` if `raw` is `0`.
    #[inline]
    pub const fn from_raw(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// Returns the number of seconds since EPOCH that this timestamp represents.
    #[inline]
    pub const fn seconds_since_epoch(self) -> u64 {
        self.0.get()
    }

    /// Returns the duration since EPOCH that this timestamp represents.
    #[inline]
    pub fn duration_since_epoch(self) -> Duration {
        Duration::from_secs(self.0.get())
    }

    /// Gets the raw, C-FFI representation of a given optional timestamp.
    ///
    /// If `timestamp` is `None`, this returns `0`.
    #[inline]
    pub fn optional_to_raw(timestamp: Option<Self>) -> u64 {
        match timestamp {
            None => 0,
            Some(t) => t.0.get(),
        }
    }
}

impl Debug for Timestamp {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.get(), f)
    }
}
