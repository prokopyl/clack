use clack_common::extensions::{Extension, HostExtensionSide};
use clap_sys::ext::thread_check::{clap_host_thread_check, CLAP_EXT_THREAD_CHECK};
use std::ffi::CStr;

#[repr(C)]
pub struct HostThreadCheck(clap_host_thread_check);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostThreadCheck {}
unsafe impl Sync for HostThreadCheck {}

unsafe impl Extension for HostThreadCheck {
    const IDENTIFIER: &'static CStr = CLAP_EXT_THREAD_CHECK;
    type ExtensionSide = HostExtensionSide;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostHandle;

    impl HostThreadCheck {
        #[inline]
        pub fn is_main_thread(&self, host: &HostHandle) -> Option<bool> {
            Some(unsafe { (self.0.is_main_thread?)(host.as_raw()) })
        }

        #[inline]
        pub fn is_audio_thread(&self, host: &HostHandle) -> Option<bool> {
            Some(unsafe { (self.0.is_audio_thread?)(host.as_raw()) })
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
        const IMPLEMENTATION: &'static Self = &HostThreadCheck(clap_host_thread_check {
            is_main_thread: Some(is_main_thread::<H>),
            is_audio_thread: Some(is_audio_thread::<H>),
        });
    }

    unsafe extern "C" fn is_main_thread<H: Host>(host: *const clap_host) -> bool
    where
        for<'a> <H as Host>::Shared<'a>: HostThreadCheckImpl,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_main_thread())).unwrap_or(false)
    }

    unsafe extern "C" fn is_audio_thread<H: Host>(host: *const clap_host) -> bool
    where
        for<'a> <H as Host>::Shared<'a>: HostThreadCheckImpl,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_audio_thread())).unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
