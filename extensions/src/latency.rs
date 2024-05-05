use clack_common::extensions::*;
use clap_sys::ext::latency::{clap_host_latency, clap_plugin_latency, CLAP_EXT_LATENCY};
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginLatency(RawExtension<PluginExtensionSide, clap_plugin_latency>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginLatency {
    const IDENTIFIER: &'static CStr = CLAP_EXT_LATENCY;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostLatency(RawExtension<HostExtensionSide, clap_host_latency>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostLatency {
    const IDENTIFIER: &'static CStr = CLAP_EXT_LATENCY;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginLatency {
        #[inline]
        pub fn get(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
            match plugin.use_extension(&self.0).get {
                None => 0,
                // SAFETY: This type ensures the function pointer is valid.
                Some(get) => unsafe { get(plugin.as_raw()) },
            }
        }
    }

    pub trait HostLatencyImpl {
        fn changed(&mut self);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostLatency
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostLatencyImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_latency {
                changed: Some(changed::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn changed<H: HostHandlers>(host: *const clap_host)
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostLatencyImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
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

    impl HostLatency {
        #[inline]
        pub fn changed(&self, host: &mut HostMainThreadHandle) {
            if let Some(changed) = host.use_extension(&self.0).changed {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { changed(host.as_raw()) }
            }
        }
    }

    pub trait PluginLatencyImpl {
        fn get(&mut self) -> u32;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginLatency
    where
        for<'a> P::MainThread<'a>: PluginLatencyImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_latency {
                get: Some(get::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get<P: Plugin>(plugin: *const clap_plugin) -> u32
    where
        for<'a> P::MainThread<'a>: PluginLatencyImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| Ok(plugin.main_thread().as_mut().get()))
            .unwrap_or(0)
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
