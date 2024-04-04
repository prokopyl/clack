use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clap_sys::ext::thread_check::{clap_host_thread_check, CLAP_EXT_THREAD_CHECK};
use std::ffi::CStr;

#[derive(Copy, Clone)]
pub struct HostThreadCheck(RawExtension<HostExtensionSide, clap_host_thread_check>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostThreadCheck {
    const IDENTIFIER: &'static CStr = CLAP_EXT_THREAD_CHECK;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostHandle;

    impl HostThreadCheck {
        #[inline]
        pub fn is_main_thread(&self, host: &HostHandle) -> Option<bool> {
            // SAFETY: This type ensures the function pointer is valid.
            Some(unsafe { host.use_extension(&self.0).is_main_thread?(host.as_raw()) })
        }

        #[inline]
        pub fn is_audio_thread(&self, host: &HostHandle) -> Option<bool> {
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

    pub trait HostThreadCheckImpl {
        fn is_main_thread(&self) -> bool;
        fn is_audio_thread(&self) -> bool;
    }

    impl<H: Host> ExtensionImplementation<H> for HostThreadCheck
    where
        for<'a> <H as Host>::Shared<'a>: HostThreadCheckImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_thread_check {
                is_main_thread: Some(is_main_thread::<H>),
                is_audio_thread: Some(is_audio_thread::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn is_main_thread<H: Host>(host: *const clap_host) -> bool
    where
        for<'a> <H as Host>::Shared<'a>: HostThreadCheckImpl,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_main_thread())).unwrap_or(false)
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn is_audio_thread<H: Host>(host: *const clap_host) -> bool
    where
        for<'a> <H as Host>::Shared<'a>: HostThreadCheckImpl,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_audio_thread())).unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
