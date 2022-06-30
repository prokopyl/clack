use clap_sys::version::clap_version;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClapVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}

impl ClapVersion {
    pub const CURRENT: ClapVersion = Self::from_raw(clap_sys::version::CLAP_VERSION);

    #[inline]
    pub const fn from_raw(raw: clap_version) -> Self {
        Self {
            major: raw.major,
            minor: raw.minor,
            revision: raw.revision,
        }
    }

    #[inline]
    pub const fn to_raw(self) -> clap_version {
        clap_version {
            major: self.major,
            minor: self.minor,
            revision: self.revision,
        }
    }

    pub const fn is_compatible(&self) -> bool {
        clap_sys::version::clap_version_is_compatible(self.to_raw())
    }
}

impl PartialOrd for ClapVersion {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClapVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.revision.cmp(&other.revision),
                o => o,
            },
            o => o,
        }
    }
}

impl Display for ClapVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
    }
}
