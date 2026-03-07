use std::ops::Add;

/// A time value in beats.
pub type BeatTime = FixedPoint;
/// A time value in seconds.
pub type SecondsTime = FixedPoint;

/// A fixed-point decimal.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedPoint(i64);

impl FixedPoint {
    /// The factor by witch to multiply a value in order to make it have the same fixed-point
    /// decimal representation as this type.
    pub const FACTOR: i64 = clap_sys::fixedpoint::CLAP_BEATTIME_FACTOR;

    /// Creates a [`FixedPoint`] decimal from its internal, raw-byte representation.
    #[inline]
    pub const fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    /// Returns the internal, raw-byte representation of this [`FixedPoint`] decimal.
    #[inline]
    pub const fn to_bits(&self) -> i64 {
        self.0
    }

    /// Creates a [`FixedPoint`] decimal from an integer value.
    #[inline]
    pub const fn from_int(val: i64) -> Self {
        Self(Self::FACTOR * val)
    }

    /// Returns the integer part of this [`FixedPoint`] decimal, discarding the decimal part.
    #[inline]
    pub const fn to_int(&self) -> i64 {
        self.0 / Self::FACTOR
    }

    /// Converts this fixed-point decimal to a floating-point value.
    #[inline]
    pub const fn to_float(&self) -> f64 {
        self.0 as f64 / Self::FACTOR as f64
    }

    /// Converts the given floating-point value to a [`FixedPoint`] decimal.
    #[inline]
    pub fn from_float(val: f64) -> Self {
        #[allow(clippy::cast_possible_truncation)] // This is willingly lossy
        Self((Self::FACTOR as f64 * val).round() as i64)
    }
}

impl From<f64> for FixedPoint {
    #[inline]
    fn from(val: f64) -> Self {
        Self::from_float(val)
    }
}

impl From<FixedPoint> for f64 {
    #[inline]
    fn from(val: FixedPoint) -> Self {
        val.to_float()
    }
}

impl Add for FixedPoint {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.0 + rhs.0)
    }
}
