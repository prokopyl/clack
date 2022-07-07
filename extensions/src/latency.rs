use clack_common::extensions::*;
use clack_host::wrapper::HostWrapper;
use clap_sys::ext::latency::{clap_host_latency, clap_plugin_latency, CLAP_EXT_LATENCY};
use std::os::raw::c_char;

#[repr(C)]
pub struct PluginLatency {
    inner: clap_plugin_latency,
}

unsafe impl Extension for PluginLatency {
    const IDENTIFIER: *const c_char = CLAP_EXT_LATENCY;
    type ExtensionType = PluginExtension;
}

#[repr(C)]
pub struct HostLatency {
    inner: clap_host_latency,
}

unsafe impl Extension for HostLatency {
    const IDENTIFIER: *const c_char = CLAP_EXT_LATENCY;
    type ExtensionType = HostExtension;
}

#[cfg(feature = "clack-host")]
const _: () = {
    use clack_host::host::PluginHoster;
    use clack_host::plugin::PluginMainThreadHandle;
    use clap_sys::host::clap_host;

    impl PluginLatency {
        #[inline]
        pub fn get(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
            unsafe { (self.inner.get)(plugin.as_raw()) }
        }
    }

    pub trait HostLatencyImpl {
        fn changed(&mut self);
    }

    impl<H: for<'a> PluginHoster<'a>> ExtensionImplementation<H> for HostLatency
    where
        for<'a> <H as PluginHoster<'a>>::MainThread: HostLatencyImpl,
    {
        const IMPLEMENTATION: &'static Self = &HostLatency {
            inner: clap_host_latency {
                changed: changed::<H>,
            },
        };
    }

    unsafe extern "C" fn changed<H: for<'a> PluginHoster<'a>>(host: *const clap_host)
    where
        for<'a> <H as PluginHoster<'a>>::MainThread: HostLatencyImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }
};

#[cfg(feature = "clack-plugin")]
const _: () = {
    use clack_plugin::host::HostMainThreadHandle;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::plugin::Plugin;
    use clap_sys::plugin::clap_plugin;

    impl HostLatency {
        #[inline]
        pub fn changed(&self, host: &mut HostMainThreadHandle) {
            unsafe { (self.inner.changed)(host.shared().as_raw()) }
        }
    }

    pub trait PluginLatencyImpl {
        fn get(&mut self) -> u32;
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginLatency
    where
        P::MainThread: PluginLatencyImpl,
    {
        const IMPLEMENTATION: &'static Self = &PluginLatency {
            inner: clap_plugin_latency { get: get::<P> },
        };
    }

    unsafe extern "C" fn get<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> u32
    where
        P::MainThread: PluginLatencyImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| Ok(plugin.main_thread().as_mut().get()))
            .unwrap_or(0)
    }
};
