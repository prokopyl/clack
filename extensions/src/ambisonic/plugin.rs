use crate::ambisonic::{AmbisonicConfig, HostAmbisonic, PluginAmbisonic};
use clack_plugin::{
    extensions::{ExtensionImplementation, wrapper::PluginWrapper},
    host::HostMainThreadHandle,
    plugin::Plugin,
};
use clap_sys::{ext::ambisonic::clap_ambisonic_config, plugin::clap_plugin};

impl HostAmbisonic {
    /// Notify the host that the ambisonic configuration for one or more ports has changed.
    ///
    /// The info can only change when the plugin is de-activated.
    pub fn changed(&self, handle: &mut HostMainThreadHandle) {
        if let Some(changed) = handle.use_extension(&self.0).changed {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { (changed)(handle.as_raw()) }
        }
    }
}

/// The plugin-side implementation of the Ambisonic extension.
pub trait PluginAmbisonicImpl {
    /// Returns true if the given configuration is supported.
    fn is_config_supported(&self, config: AmbisonicConfig) -> bool;

    /// Returns the ambisonic configuration for the given port, if applicable.
    fn get_config(&self, is_input: bool, port_index: u32) -> Option<AmbisonicConfig>;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginAmbisonic
where
    for<'a> P: Plugin<MainThread<'a>: PluginAmbisonicImpl>,
{
    const IMPLEMENTATION: clack_plugin::extensions::RawExtensionImplementation =
        clack_plugin::extensions::RawExtensionImplementation::new(
            &clap_sys::ext::ambisonic::clap_plugin_ambisonic {
                is_config_supported: Some(is_config_supported::<P>),
                get_config: Some(get_config::<P>),
            },
        );
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn is_config_supported<P>(
    plugin: *const clap_plugin,
    config: *const clap_ambisonic_config,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginAmbisonicImpl>,
{
    unsafe {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            let config = AmbisonicConfig::from_raw(*config);

            Ok(plugin.main_thread().as_ref().is_config_supported(config))
        })
        .unwrap_or(false)
    }
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn get_config<P>(
    plugin: *const clap_plugin,
    is_input: bool,
    port_index: u32,
    config: *mut clap_ambisonic_config,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginAmbisonicImpl>,
{
    unsafe {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            match plugin
                .main_thread()
                .as_ref()
                .get_config(is_input, port_index)
            {
                Some(output) => {
                    config.write(*output.as_raw());
                    Ok(true)
                }
                None => Ok(false),
            }
        })
        .unwrap_or(false)
    }
}
