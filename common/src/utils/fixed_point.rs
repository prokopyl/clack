use std::ops::Add;

pub type BeatTime = FixedPoint;
pub type SecondsTime = FixedPoint;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FixedPoint(i64);

impl FixedPoint {
    pub const FACTOR: i64 = clap_sys::fixedpoint::CLAP_BEATTIME_FACTOR;

    #[inline]
    pub const fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn to_bits(&self) -> i64 {
        self.0
    }

    #[inline]
    pub const fn from_int(val: i64) -> Self {
        Self(Self::FACTOR * val)
    }

    #[inline]
    pub const fn to_int(&self) -> i64 {
        self.0 / Self::FACTOR
    }

    #[inline]
    pub fn to_float(&self) -> f64 {
        self.0 as f64 / Self::FACTOR as f64
    }

    #[inline]
    pub fn from_float(val: f64) -> Self {
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
