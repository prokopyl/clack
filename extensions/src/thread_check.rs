//! Allows plugins to check which threads their functions are being called on.

use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clap_sys::ext::thread_check::{CLAP_EXT_THREAD_CHECK, clap_host_thread_check};
use std::ffi::CStr;

/// Host-side of the Thread Check extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostThreadCheck(RawExtension<HostExtensionSide, clap_host_thread_check>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostThreadCheck {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_THREAD_CHECK];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostSharedHandle;

    impl HostThreadCheck {
        /// Returns `true` if the current thread is the main thread, and `false` if it is not.
        ///
        /// This may return `None` in case the host hasn't properly implemented this call.
        #[inline]
        pub fn is_main_thread(&self, host: &HostSharedHandle) -> Option<bool> {
            // SAFETY: This type ensures the function pointer is valid.
            Some(unsafe { host.use_extension(&self.0).is_main_thread?(host.as_raw()) })
        }

        /// Returns `true` if the current thread is the audio thread, and `false` if it is not.
        ///
        /// Note that the current thread can both be the main thread and audio thread, in e.g.
        /// non-realtime contexts.
        ///
        /// This may return `None` in case the host hasn't properly implemented this call.
        #[inline]
        pub fn is_audio_thread(&self, host: &HostSharedHandle) -> Option<bool> {
            // SAFETY: This type ensures the function pointer is valid.
            Some(unsafe { host.use_extension(&self.0).is_audio_thread?(host.as_raw()) })
        }
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use crate::thread_check::HostThreadCheck;
    use clack_host::extensions::prelude::*;
    use clap_sys::ext::thread_check::clap_host_thread_check;

    /// Implementation of the Host-side of the Thread Check extension.
    pub trait HostThreadCheckImpl {
        /// Returns `true` if the current thread is the main thread, and `false` if it is not.
        ///
        /// This may return `None` in case the host hasn't properly implemented this call.
        fn is_main_thread(&self) -> bool;

        /// Returns `true` if the current thread is the audio thread, and `false` if it is not.
        ///
        /// Note that the current thread can both be the main thread and audio thread, in e.g.
        /// non-realtime contexts.
        ///
        /// This may return `None` in case the host hasn't properly implemented this call.
        fn is_audio_thread(&self) -> bool;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostThreadCheck
    where
        for<'a> H: HostHandlers<Shared<'a>: HostThreadCheckImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_thread_check {
                is_main_thread: Some(is_main_thread::<H>),
                is_audio_thread: Some(is_audio_thread::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn is_main_thread<H>(host: *const clap_host) -> bool
    where
        for<'a> H: HostHandlers<Shared<'a>: HostThreadCheckImpl>,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_main_thread())).unwrap_or(false)
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn is_audio_thread<H>(host: *const clap_host) -> bool
    where
        for<'a> H: HostHandlers<Shared<'a>: HostThreadCheckImpl>,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_audio_thread())).unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
