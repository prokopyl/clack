pub mod implementation {
    use crate::factory::FactoryImplementation;
    use crate::host::HostInfo;
    use crate::plugin::{PluginDescriptor, PluginInstance};
    use clap_sys::host::clap_host;
    use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
    use clap_sys::plugin_factory::clap_plugin_factory;
    use std::ffi::CStr;

    pub trait PluginFactory<'a> {
        fn plugin_count() -> u32;
        fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor>;
        fn create_plugin(host_info: HostInfo<'a>, plugin_id: &[u8]) -> Option<PluginInstance<'a>>;
    }

    impl<'a, F: PluginFactory<'a>> FactoryImplementation<F>
        for clack_common::factory::plugin::PluginFactory
    {
        const IMPLEMENTATION: &'static Self = &clack_common::factory::plugin::PluginFactory {
            inner: clap_plugin_factory {
                get_plugin_count: get_plugin_count::<F>,
                get_plugin_descriptor: get_plugin_descriptor::<F>,
                create_plugin: create_plugin::<F>,
            },
        };
    }

    unsafe extern "C" fn get_plugin_count<'a, E: PluginFactory<'a>>(
        _f: *const clap_plugin_factory,
    ) -> u32 {
        E::plugin_count()
    }

    unsafe extern "C" fn get_plugin_descriptor<'a, E: PluginFactory<'a>>(
        _f: *const clap_plugin_factory,
        index: u32,
    ) -> *const clap_plugin_descriptor {
        match E::plugin_descriptor(index) {
            None => core::ptr::null(),
            Some(d) => &d.0,
        }
    }

    unsafe extern "C" fn create_plugin<'a, E: PluginFactory<'a>>(
        _f: *const clap_plugin_factory,
        clap_host: *const clap_host,
        plugin_id: *const std::os::raw::c_char,
    ) -> *const clap_plugin {
        let plugin_id = CStr::from_ptr(plugin_id).to_bytes_with_nul();
        let clap_host = if let Some(clap_host) = clap_host.as_ref() {
            clap_host
        } else {
            eprintln!("[ERROR] Null clap_host pointer was provided to entry::create_plugin.");
            return core::ptr::null();
        };

        let host_info = HostInfo::from_raw(clap_host);

        match E::create_plugin(host_info, plugin_id) {
            None => core::ptr::null(),
            Some(instance) => instance.into_owned_ptr(),
        }
    }
}
