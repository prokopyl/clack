use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

/// A hint that indicates which channels of an audio port are constant.
///
/// Channels are defined as constant when all of their samples have the exact same value. This
/// however doesn't mean they are _silent_: the value could be different from `0`.
///
/// This type is internally by a 64-bit bitmask, as per the CLAP specification, hence its name. This
/// also means a constant mask can only hold constant hints for up to 64 channels: information about
/// any extra channels is lost, and this implementation considers those channels to never be
/// constant.
///
/// # Example
///
/// ```
/// use clack_common::process::ConstantMask;
///
/// let mut constant_mask = ConstantMask::from_bits(0b101);
///
/// assert!(constant_mask.is_channel_constant(0));
/// assert!(!constant_mask.is_channel_constant(1));
/// assert!(constant_mask.is_channel_constant(2));
///
/// constant_mask.set_channel_constant(1, true);
/// assert!(constant_mask.is_channel_constant(1));
/// assert_eq!(0b111, constant_mask.to_bits());
///
/// ```
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ConstantMask(u64);

impl ConstantMask {
    pub const CAPACITY: u8 = 64;

    pub const FULLY_CONSTANT: ConstantMask = ConstantMask(u64::MAX);
    pub const FULLY_DYNAMIC: ConstantMask = ConstantMask(0);

    /// Creates a new constant mask from its inner bitmask representation.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        ConstantMask(bits)
    }

    /// Gets the constant mask's inner bitmask representation.
    #[inline]
    pub const fn to_bits(&self) -> u64 {
        self.0
    }

    /// Returns an iterator over the constant status of each channel in this constant mask.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::process::ConstantMask;
    ///
    /// let mut iter = ConstantMask::from_bits(0b101).iter();
    ///
    /// assert_eq!(Some(true), iter.next());
    /// assert_eq!(Some(false), iter.next());
    /// assert_eq!(Some(true), iter.next());
    /// assert_eq!(Some(false), iter.next());
    /// assert_eq!(Some(false), iter.next());
    /// ```
    #[inline]
    pub const fn iter(&self) -> ConstantMaskIter {
        ConstantMaskIter(self.0)
    }

    /// Returns `true` if the channel at the given index is constant, `false` otherwise.
    ///
    /// This function will always return `false` when given any index over `63`.
    #[inline]
    pub const fn is_channel_constant(&self, channel_index: u64) -> bool {
        if channel_index > 63 {
            return false;
        }

        (self.0 & (1 << channel_index)) != 0
    }

    /// Sets whether the channel at the given index is constant.
    ///
    /// This function do nothing when given any index over `63`.
    #[inline]
    pub fn set_channel_constant(&mut self, channel_index: u64, value: bool) {
        if channel_index > 63 {
            return;
        }

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

impl Default for ConstantMask {
    /// Returns an empty constant mask, i.e. one where every channel is considered dynamic.
    #[inline]
    fn default() -> Self {
        ConstantMask::FULLY_DYNAMIC
    }
}

impl IntoIterator for ConstantMask {
    type Item = bool;
    type IntoIter = ConstantMaskIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &ConstantMask {
    type Item = bool;
    type IntoIter = ConstantMaskIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &mut ConstantMask {
    type Item = bool;
    type IntoIter = ConstantMaskIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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

/// An iterator over the constant status of each channel in a [`ConstantMask`].
///
/// This iterator is infinite: once all channels have been iterated over, it will return `false`
/// continuously. Use it alongside a limiting iterator adapter, such as [`zip`](Iterator::zip) or
/// [`take`](Iterator::take).
///
/// # Example
///
/// ```
/// use clack_common::process::ConstantMask;
///
/// let mut iter = ConstantMask::from_bits(0b101).iter();
///
/// assert_eq!(Some(true), iter.next());
/// assert_eq!(Some(false), iter.next());
/// assert_eq!(Some(true), iter.next());
/// assert_eq!(Some(false), iter.next());
/// assert_eq!(Some(false), iter.next());
/// ```
#[derive(Copy, Clone)]
pub struct ConstantMaskIter(u64);

impl Iterator for ConstantMaskIter {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let is_constant = self.0 & 1 != 0;
        self.0 >>= 1;
        Some(is_constant)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<bool> {
        self.0 >>= n;

        self.next()
    }
}
