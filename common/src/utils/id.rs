use core::num::NonZeroU32;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};

/// A standardized CLAP identifier.
///
/// Identifiers in CLAP are 32-bit unsigned integers, but where the [`u32::MAX`] value is invalid.
///
/// This invalid value is generally used in the CLAP ABI to indicate failure.
///
/// This type ensures that the invalid value (i.e. [`u32::MAX`]) is never used in place of an actual
/// CLAP ID.
///
/// # Example
///
/// ```
/// use clack_common::utils::ClapId;
///
/// let id: ClapId = ClapId::new(42);
/// assert_eq!(42, id.get());
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ClapId(NonZeroU32);

impl ClapId {
    /// Creates a new CLAP identifier from its numeric value.
    ///
    /// # Panics
    ///
    /// This function will panic if `id` is [`u32::MAX`].
    ///
    /// For a non-panicking version, see [`ClapId::from_raw`].
    #[inline]
    pub const fn new(id: u32) -> Self {
        match Self::from_raw(id) {
            None => panic!("Invalid ClapId"),
            Some(id) => id,
        }
    }

    /// Creates a new CLAP identifier from its raw numeric value.
    ///
    /// Returns `None` if the given `id` is [`u32::MAX`].
    ///
    /// For a panicking version that returns the [`ClapId`] directly, see [`ClapId::new`].
    #[inline]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match NonZeroU32::new(raw.wrapping_add(1)) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// Gets the underlying numerical value of this ID.
    ///
    /// The returned value is guaranteed to never be [`u32::MAX`].
    #[inline]
    pub const fn get(self) -> u32 {
        self.0.get().wrapping_sub(1)
    }

    /// Takes an optional identifier, and returns the matching raw, C-FFI compatible value.
    ///
    /// This returns the underlying numerical value of the ID if present, or [`u32::MAX`] if `None`.
    #[inline]
    pub const fn optional_to_raw(value: Option<ClapId>) -> u32 {
        match value {
            None => u32::MAX,
            Some(value) => value.get(),
        }
    }
}

impl Debug for ClapId {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ClapId").field(&self.get()).finish()
    }
}

impl Display for ClapId {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl From<u32> for ClapId {
    #[inline]
    fn from(value: u32) -> Self {
        ClapId::new(value)
    }
}

impl From<ClapId> for u32 {
    #[inline]
    fn from(value: ClapId) -> Self {
        value.get()
    }
}

impl PartialEq<u32> for ClapId {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl PartialEq<ClapId> for u32 {
    #[inline]
    fn eq(&self, other: &ClapId) -> bool {
        *self == other.get()
    }
}

impl PartialOrd<u32> for ClapId {
    #[inline]
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl PartialOrd<ClapId> for u32 {
    #[inline]
    fn partial_cmp(&self, other: &ClapId) -> Option<Ordering> {
        self.partial_cmp(&other.get())
    }
}

impl PartialEq<Option<ClapId>> for ClapId {
    #[inline]
    fn eq(&self, other: &Option<ClapId>) -> bool {
        match other {
            None => false,
            Some(other) => self.0 == other.0,
        }
    }
}

impl PartialEq<ClapId> for Option<ClapId> {
    #[inline]
    fn eq(&self, other: &ClapId) -> bool {
        match self {
            None => false,
            Some(s) => s.0 == other.0,
        }
    }
}
