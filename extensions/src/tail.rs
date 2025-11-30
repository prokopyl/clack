//! Allows plugins to communicate their tail length to the host.

#![deny(missing_docs)]

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::tail::*;
use std::ffi::CStr;

/// The Plugin-side of the Tail extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginTail(RawExtension<PluginExtensionSide, clap_plugin_tail>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginTail {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TAIL;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The Host-side of the Tail extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostTail(RawExtension<HostExtensionSide, clap_host_tail>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostTail {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TAIL;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The length of a plugin's tail, which can potentially be infinite.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum TailLength {
    /// The tail is finite and has a given length, in samples
    Finite(u32),
    /// The tail is infinite
    Infinite,
}

/// The Default for a tail length is no tail at all, i.e. a finite tail of length `0`.
impl Default for TailLength {
    #[inline]
    fn default() -> Self {
        Self::Finite(0)
    }
}

impl TailLength {
    /// Returns the tail length corresponding to the given raw C FFI-compatible value.
    ///
    /// Any value superior or equal to [`i32::MAX`] will be considered infinite.
    pub const fn from_raw(raw: u32) -> Self {
        if raw >= i32::MAX as u32 {
            Self::Infinite
        } else {
            Self::Finite(raw)
        }
    }

    /// Returns the tail length as a raw C FFI-compatible value.
    ///
    /// [`i32::MAX`] will be returned if the tail is infinite.
    pub const fn to_raw(&self) -> u32 {
        match self {
            TailLength::Finite(length) => *length,
            TailLength::Infinite => i32::MAX as u32,
        }
    }

    /// Returns if it has any tail.
    ///
    /// Returns `false` for a finite tail of length `0`, and `true` otherwise.
    #[inline]
    pub const fn has_tail(&self) -> bool {
        !matches!(self, TailLength::Finite(0))
    }

    /// Returns if this tail's length is finite.
    #[inline]
    pub const fn is_finite(&self) -> bool {
        match self {
            TailLength::Finite(_) => true,
            TailLength::Infinite => false,
        }
    }

    /// Returns if this tail's length is infinite.
    #[inline]
    pub const fn is_infinite(&self) -> bool {
        !self.is_finite()
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginTail {
        /// Returns the plugin's [`TailLength`].
        #[inline]
        pub fn get(&self, plugin: &PluginAudioProcessorHandle) -> TailLength {
            match plugin.use_extension(&self.0).get {
                // SAFETY: This type ensures the function pointer is valid.
                Some(get) => TailLength::from_raw(unsafe { get(plugin.as_raw()) }),
                None => TailLength::default(),
            }
        }
    }

    /// Implementation of the Host-side of the Tail extension.
    pub trait HostTailImpl {
        /// Informs the host that the plugin's tail length has changed and needs to be updated.
        fn changed(&mut self);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostTail
    where
        H: for<'a> HostHandlers<AudioProcessor<'a>: HostTailImpl>,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_tail {
                changed: Some(changed::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn changed<H>(host: *const clap_host)
    where
        for<'a> H: HostHandlers<AudioProcessor<'a>: HostTailImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            host.audio_processor()?.as_mut().changed();
            Ok(())
        });
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    impl HostTail {
        /// Informs the host that the plugin's tail length has changed and needs to be updated.
        #[inline]
        pub fn changed(&self, host: &mut HostAudioProcessorHandle) {
            if let Some(changed) = host.use_extension(&self.0).changed {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { changed(host.as_raw()) }
            }
        }
    }

    /// Implementation of the Plugin-side of the Tail extension.
    pub trait PluginTailImpl {
        /// Returns the plugin's [`TailLength`].
        fn get(&self) -> TailLength;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P> ExtensionImplementation<P> for PluginTail
    where
        for<'a> P: Plugin<AudioProcessor<'a>: PluginTailImpl>,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_tail {
                get: Some(get::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get<P>(plugin: *const clap_plugin) -> u32
    where
        for<'a> P: Plugin<AudioProcessor<'a>: PluginTailImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            Ok(plugin.audio_processor()?.as_ref().get().to_raw())
        })
        .unwrap_or_else(|| TailLength::default().to_raw())
    }
}

#[cfg(feature = "clack-plugin")]
pub use plugin::*;
