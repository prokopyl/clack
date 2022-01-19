use core::num::NonZeroU32;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct ClapId(NonZeroU32);

impl ClapId {
    #[inline]
    pub const fn new(raw: u32) -> Self {
        match Self::from_raw(raw) {
            None => panic!("Invalid ClapId"),
            Some(id) => id,
        }
    }

    #[inline]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match NonZeroU32::new(raw.wrapping_add(1)) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.0.get().wrapping_sub(1)
    }

    #[inline]
    pub const fn optional_to_raw(value: Option<ClapId>) -> u32 {
        match value {
            None => u32::MAX,
            Some(value) => value.get(),
        }
    }
}

impl From<u32> for ClapId {
    fn from(value: u32) -> Self {
        ClapId::new(value)
    }
}
