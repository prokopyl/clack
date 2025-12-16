use std::num::NonZeroU64;
use std::time::Duration;

// TODO: Debug impl
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct Timestamp(NonZeroU64);

impl Timestamp {
    #[inline]
    pub fn from_raw(raw: u64) -> Option<Self> {
        NonZeroU64::new(raw).map(Self)
    }

    #[inline]
    pub fn seconds_since_epoch(self) -> u64 {
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
