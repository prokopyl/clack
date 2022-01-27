#[cfg(feature = "clack-plugin")]
pub trait PluginLatency {
    fn latency(&mut self) -> u32;
}

#[cfg(feature = "clack-plugin")]
const _: () = {
    use clack_plugin::extensions::ExtensionImplementation;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::prelude::Plugin;
    use clap_sys::ext::latency::clap_plugin_latency;
    use clap_sys::plugin::clap_plugin;

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for super::PluginLatency
    where
        P::MainThread: PluginLatency,
    {
        const IMPLEMENTATION: &'static Self = &Self {
            inner: clap_plugin_latency { get: get::<P> },
        };
    }

    unsafe extern "C" fn get<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> u32
    where
        P::MainThread: PluginLatency,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| Ok(plugin.main_thread().as_mut().latency()))
            .unwrap_or(0)
    }
};

#[cfg(feature = "clack-host")]
pub trait HostLatency {
    fn changed(&mut self);
}

#[cfg(feature = "clack-host")]
const _: () = {
    use clack_host::extensions::ExtensionImplementation;
    use clack_host::host::PluginHoster;
    use clack_host::wrapper::HostWrapper;
    use clap_sys::ext::latency::clap_host_latency;
    use clap_sys::host::clap_host;

    impl<'a, H: PluginHoster<'a>> ExtensionImplementation<H> for super::HostLatency
    where
        H: HostLatency,
    {
        const IMPLEMENTATION: &'static Self = &Self {
            inner: clap_host_latency {
                changed: changed::<H>,
            },
        };
    }

    unsafe extern "C" fn changed<'a, H: PluginHoster<'a> + HostLatency>(host: *const clap_host) {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }
};
