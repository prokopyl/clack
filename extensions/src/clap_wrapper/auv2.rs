//! This module implements the AUv2 plugin info extension of the clap-wrapper project.
//! Using these extensions, we can tell the wrapper how to advertise our CLAP plugins as AUv2.

mod sys;
use sys::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[derive(Debug, Copy, Clone)]
pub struct PluginInfoAsAUv2 {
    inner: clap_plugin_info_as_auv2,
}

impl PluginInfoAsAUv2 {
    pub(crate) fn empty() -> PluginInfoAsAUv2 {
        Self {
            inner: clap_plugin_info_as_auv2 {
                au_subt: [0; 5],
                au_type: [0; 5],
            },
        }
    }

    #[inline]
    pub fn new(au_type: &str, au_subt: &str) -> Self {
        assert_eq!(
            au_type.len(),
            4,
            "au_type must be exactly 4 characters long"
        );
        assert_eq!(
            au_subt.len(),
            4,
            "au_subt must be exactly 4 characters long"
        );

        let mut inner = clap_plugin_info_as_auv2 {
            au_type: [0; 5],
            au_subt: [0; 5],
        };

        inner.au_type[..4].copy_from_slice(au_type.as_bytes());
        inner.au_subt[..4].copy_from_slice(au_subt.as_bytes());

        // Byte 4 is already zero due to array init: [0; 5]

        Self { inner }
    }

    #[inline]
    pub const fn au_type(&self) -> &[u8; 5] {
        &self.inner.au_type
    }

    #[inline]
    pub const fn au_subt(&self) -> &[u8; 5] {
        &self.inner.au_subt
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_plugin_info_as_auv2 {
        &self.inner
    }

    #[inline]
    pub const fn as_raw_mut(&mut self) -> &mut clap_plugin_info_as_auv2 {
        &mut self.inner
    }
}
