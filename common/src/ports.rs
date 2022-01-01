use clap_sys::chmap::*;

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum ChannelMap {
    Unspecified = CLAP_CHMAP_UNSPECIFIED,
    Mono = CLAP_CHMAP_MONO,
    Stereo = CLAP_CHMAP_STEREO,
    Surround = CLAP_CHMAP_SURROUND,
}

impl ChannelMap {
    pub fn from_raw(raw: clap_chmap) -> Self {
        use ChannelMap::*;

        match raw {
            CLAP_CHMAP_MONO => Mono,
            CLAP_CHMAP_STEREO => Stereo,
            CLAP_CHMAP_SURROUND => Surround,
            _ => Unspecified,
        }
    }

    #[inline]
    pub fn to_raw(&self) -> clap_chmap {
        *self as clap_chmap
    }
}
