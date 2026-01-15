use crate::configurable_audio_ports::{AudioPortsRequestList, PluginConfigurableAudioPorts};
use clack_common::extensions::{ExtensionImplementation, RawExtensionImplementation};
use clack_plugin::{
    extensions::wrapper::{PluginWrapper, PluginWrapperError},
    plugin::Plugin,
};
use clap_sys::{
    ext::configurable_audio_ports::{
        clap_audio_port_configuration_request, clap_plugin_configurable_audio_ports,
    },
    plugin::clap_plugin,
};

/// Implementation of the Plugin-side of the Configurable Audio Ports extension.
pub trait PluginConfigurableAudioPortsImpl {
    /// Returns true if the given configurations can be applied using [`apply_configuration`](PluginConfigurableAudioPortsImpl::apply_configuration).
    ///
    /// Must be called when the plugin is deactivated.
    fn can_apply_configuration(&mut self, list: AudioPortsRequestList<'_>) -> bool;

    /// Submit a bunch of configuration requests which will atomically be applied together,
    /// or discarded together.
    ///
    /// Once the configuration is successfully applied, it isn't necessary for the plugin to call
    /// [`HostAudioPorts::rescan`](crate::audio_ports::HostAudioPorts::rescan); and it isn't necessary for the host to scan the
    /// audio ports.
    ///
    /// Returns true if applied, false otherwise.
    ///
    /// Must be called when the plugin is deactivated.
    fn apply_configuration(&mut self, list: AudioPortsRequestList<'_>) -> bool;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginConfigurableAudioPorts
where
    for<'a> P: Plugin<MainThread<'a>: PluginConfigurableAudioPortsImpl>,
{
    #[doc(hidden)]
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_configurable_audio_ports {
            can_apply_configuration: Some(can_apply_configuration::<P>),
            apply_configuration: Some(apply_configuration::<P>),
        });
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn can_apply_configuration<P>(
    plugin: *const clap_plugin,
    requests: *const clap_audio_port_configuration_request,
    count: u32,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginConfigurableAudioPortsImpl>,
{
    unsafe {
        PluginWrapper::<P>::handle(plugin, |p| {
            if p.is_active() {
                return Err(PluginWrapperError::DeactivationRequiredForFunction(
                    "clap_plugin_configurable_audio_ports.can_apply_configuration",
                ));
            }

            Ok(p.main_thread()
                .as_mut()
                .can_apply_configuration(AudioPortsRequestList::from_raw(requests, count)))
        })
        .unwrap_or(false)
    }
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn apply_configuration<P>(
    plugin: *const clap_plugin,
    requests: *const clap_audio_port_configuration_request,
    count: u32,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginConfigurableAudioPortsImpl>,
{
    unsafe {
        PluginWrapper::<P>::handle(plugin, |p| {
            if p.is_active() {
                return Err(PluginWrapperError::DeactivationRequiredForFunction(
                    "clap_plugin_configurable_audio_ports.apply_configuration",
                ));
            }

            Ok(p.main_thread()
                .as_mut()
                .apply_configuration(AudioPortsRequestList::from_raw(requests, count)))
        })
        .unwrap_or(false)
    }
}
