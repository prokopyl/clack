use clack_common::extensions::{Extension, HostExtension};
use clap_sys::ext::thread_check::{clap_host_thread_check, CLAP_EXT_THREAD_CHECK};
use std::os::raw::c_char;

#[repr(C)]
pub struct ThreadCheck(clap_host_thread_check);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for ThreadCheck {}
unsafe impl Sync for ThreadCheck {}

unsafe impl Extension for ThreadCheck {
    const IDENTIFIER: *const c_char = CLAP_EXT_THREAD_CHECK;
    type ExtensionType = HostExtension;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostHandle;

    impl ThreadCheck {
        #[inline]
        pub fn is_main_thread(&self, host: &HostHandle) -> bool {
            unsafe { (self.0.is_main_thread)(host.as_raw()) }
        }

        #[inline]
        pub fn is_audio_thread(&self, host: &HostHandle) -> bool {
            unsafe { (self.0.is_main_thread)(host.as_raw()) }
        }
    }
}

#[cfg(feature = "clack-host")]
pub mod host {
    use crate::thread_check::ThreadCheck;
    use clack_common::extensions::ExtensionImplementation;
    use clack_host::host::PluginHoster;
    use clack_host::wrapper::HostWrapper;
    use clap_sys::ext::thread_check::clap_host_thread_check;
    use clap_sys::host::clap_host;

    pub trait ThreadCheckImplementation {
        fn is_main_thread(&self) -> bool;
        fn is_audio_thread(&self) -> bool;
    }

    impl<H: for<'a> PluginHoster<'a>> ExtensionImplementation<H> for ThreadCheck
    where
        for<'a> <H as PluginHoster<'a>>::Shared: ThreadCheckImplementation,
    {
        const IMPLEMENTATION: &'static Self = &ThreadCheck(clap_host_thread_check {
            is_main_thread: is_main_thread::<H>,
            is_audio_thread: is_audio_thread::<H>,
        });
    }

    unsafe extern "C" fn is_main_thread<H: for<'a> PluginHoster<'a>>(host: *const clap_host) -> bool
    where
        for<'a> <H as PluginHoster<'a>>::Shared: ThreadCheckImplementation,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_main_thread())).unwrap_or(false)
    }

    unsafe extern "C" fn is_audio_thread<H: for<'a> PluginHoster<'a>>(
        host: *const clap_host,
    ) -> bool
    where
        for<'a> <H as PluginHoster<'a>>::Shared: ThreadCheckImplementation,
    {
        HostWrapper::<H>::handle(host, |host| Ok(host.shared().is_main_thread())).unwrap_or(false)
    }
}
