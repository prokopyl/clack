use crate::ambisonic::{AmbisonicConfig, HostAmbisonic, PluginAmbisonic};
use clack_host::{
    extensions::{ExtensionImplementation, RawExtensionImplementation, wrapper::HostWrapper},
    host::HostHandlers,
    plugin::PluginMainThreadHandle,
};
use clap_sys::{
    ext::ambisonic::{clap_ambisonic_config, clap_host_ambisonic},
    host::clap_host,
};

impl PluginAmbisonic {
    /// Check if the plugin supports the given ambisonic configuration.
    pub fn is_config_supported(
        &self,
        handle: &mut PluginMainThreadHandle,
        config: AmbisonicConfig,
    ) -> bool {
        if let Some(is_config_supported) = handle.use_extension(&self.0).is_config_supported {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { (is_config_supported)(handle.as_raw_ptr(), config.as_raw()) }
        } else {
            false
        }
    }

    /// Get the ambisonic configuration for the given port, if applicable.
    pub fn get_config(
        &self,
        handle: &mut PluginMainThreadHandle,
        is_input: bool,
        port_index: u32,
    ) -> Option<AmbisonicConfig> {
        if let Some(get_config) = handle.use_extension(&self.0).get_config {
            let mut config = clap_ambisonic_config {
                ordering: 0,
                normalization: 0,
            };

            // SAFETY: This type ensures the function pointer is valid.
            let result =
                unsafe { (get_config)(handle.as_raw(), is_input, port_index, &mut config) };

            result.then(|| AmbisonicConfig::from_raw(config))
        } else {
            None
        }
    }
}

/// The host-side implementation of the Ambisonic extension.
pub trait HostAmbisonicImpl {
    /// Notify the host that the ambisonic configuration for one or more ports has changed.
    ///
    /// The info can only change when the plugin is de-activated.
    fn changed(&mut self);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostAmbisonic
where
    for<'a> H: HostHandlers<MainThread<'a>: HostAmbisonicImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_ambisonic {
            changed: Some(changed::<H>),
        });
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn changed<H>(host: *const clap_host)
where
    for<'a> H: HostHandlers<MainThread<'a>: HostAmbisonicImpl>,
{
    unsafe {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }
}
