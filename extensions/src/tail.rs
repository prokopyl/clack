//! Allows plugins to communicate their tail length to the host.

#![deny(missing_docs)]

use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::tail::*;
use std::ffi::CStr;

/// The Plugin-side of the Tail extension.
#[repr(C)]
pub struct PluginTail(clap_plugin_tail);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginTail {}
unsafe impl Sync for PluginTail {}

unsafe impl Extension for PluginTail {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TAIL;
    type ExtensionType = PluginExtension;
}

/// The Host-side of the Tail extension.
#[repr(C)]
pub struct HostTail(clap_host_tail);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostTail {}
unsafe impl Sync for HostTail {}

unsafe impl Extension for HostTail {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TAIL;
    type ExtensionType = HostExtension;
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
    use clack_common::extensions::ExtensionImplementation;
    use clack_host::host::Host;
    use clack_host::plugin::PluginAudioProcessorHandle;
    use clack_host::wrapper::HostWrapper;
    use clap_sys::host::clap_host;

    impl PluginTail {
        /// Returns the plugin's [`TailLength`].
        #[inline]
        pub fn get(&self, plugin: &PluginAudioProcessorHandle) -> TailLength {
            match self.0.get {
                Some(get) => TailLength::from_raw(unsafe { get(plugin.as_raw()) }),
                None => TailLength::default(),
            }
        }
    }

    /// Implementation of the Host-side of the Tail extension.
    pub trait HostTailImplementation {
        /// Informs the host that the plugin's tail length has changed and needs to be updated.
        fn changed(&mut self);
    }

    impl<H: for<'a> Host<'a>> ExtensionImplementation<H> for HostTail
    where
        for<'a> <H as Host<'a>>::AudioProcessor: HostTailImplementation,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_host_tail {
            changed: Some(changed::<H>),
        });
    }

    unsafe extern "C" fn changed<H: for<'a> Host<'a>>(host: *const clap_host)
    where
        for<'a> <H as Host<'a>>::AudioProcessor: HostTailImplementation,
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
    use clack_common::extensions::ExtensionImplementation;
    use clack_plugin::host::HostAudioThreadHandle;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::prelude::Plugin;
    use clap_sys::plugin::clap_plugin;

    impl HostTail {
        /// Informs the host that the plugin's tail length has changed and needs to be updated.
        #[inline]
        pub fn changed(&self, host: &mut HostAudioThreadHandle) {
            if let Some(changed) = self.0.changed {
                unsafe { changed(host.as_raw()) }
            }
        }
    }

    /// Implementation of the Plugin-side of the Tail extension.
    pub trait PluginTailImplementation {
        /// Returns the plugin's [`TailLength`].
        fn get(&self) -> TailLength;
    }

    impl<P: for<'a> Plugin<'a>> ExtensionImplementation<P> for PluginTail
    where
        for<'a> P: PluginTailImplementation,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_plugin_tail {
            get: Some(get::<P>),
        });
    }

    unsafe extern "C" fn get<P: for<'a> Plugin<'a>>(plugin: *const clap_plugin) -> u32
    where
        for<'a> P: PluginTailImplementation,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            Ok(plugin.audio_processor()?.as_ref().get().to_raw())
        })
        .unwrap_or_else(|| TailLength::default().to_raw())
    }
}

#[cfg(feature = "clack-plugin")]
pub use plugin::*;
