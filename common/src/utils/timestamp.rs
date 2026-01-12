use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU64;
use std::time::Duration;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Timestamp(NonZeroU64);

impl Timestamp {
    #[inline]
    pub const fn from_raw(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    #[inline]
    pub const fn seconds_since_epoch(self) -> u64 {
        self.0.get()
    }

    #[inline]
    pub fn duration_since_epoch(self) -> Duration {
        Duration::from_secs(self.0.get())
    }

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
