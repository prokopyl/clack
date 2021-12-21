use NoteEventMatch::*;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum NoteEventMatch<T: Copy + Ord + Eq + TryFrom<i32> + Into<i32> = u8> {
    Specific(T),
    All,
    None,
}

impl<T: Copy + Ord + Eq + TryFrom<i32> + Into<i32>> NoteEventMatch<T> {
    #[inline]
    pub fn from_raw(raw: i32) -> Self {
        match raw {
            -1 => All,
            raw => raw.try_into().map(Specific).unwrap_or(None),
        }
    }

    #[inline]
    pub fn to_raw(self) -> i32 {
        match self {
            Specific(val) => val.into(),
            All => -1,
            None => i32::MAX,
        }
    }
}
