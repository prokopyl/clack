use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::latency::{clap_host_latency, clap_plugin_latency, CLAP_EXT_LATENCY};

pub mod implementation;

#[repr(C)]
pub struct PluginLatency {
    inner: clap_plugin_latency,
}

unsafe impl Extension for PluginLatency {
    const IDENTIFIER: *const u8 = CLAP_EXT_LATENCY.cast();
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
const _: () = {
    use clack_host::plugin::PluginMainThread;

    impl PluginLatency {
        #[inline]
        pub fn get(&self, plugin: &mut PluginMainThread) -> u32 {
            unsafe { (self.inner.get)(plugin.as_raw()) }
        }
    }
};

#[repr(C)]
pub struct HostLatency {
    inner: clap_host_latency,
}

unsafe impl Extension for HostLatency {
    const IDENTIFIER: *const u8 = CLAP_EXT_LATENCY.cast();
    type ExtensionType = HostExtension;
}

#[cfg(feature = "clack-plugin")]
const _: () = {
    use clack_plugin::host::HostMainThreadHandle;

    impl HostLatency {
        #[inline]
        pub fn changed(&self, host: &mut HostMainThreadHandle) {
            unsafe { (self.inner.changed)(host.shared().as_raw()) }
        }
    }
};
