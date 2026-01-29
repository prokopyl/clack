use crate::{
    audio_ports::AudioPortType,
    configurable_audio_ports::{
        AudioPortsRequest, AudioPortsRequestPort, PluginConfigurableAudioPorts,
    },
};
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
use std::fmt::{self, Debug};

/// A list of [`AudioPortsRequest`]s.
#[derive(Copy, Clone)]
pub struct AudioPortsRequestList<'a> {
    raw: &'a [clap_audio_port_configuration_request],
}

impl<'a> AudioPortsRequestList<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is valid for the duration of lifetime `'a`,
    /// and that it points to an array of at least `len` elements.
    unsafe fn from_raw(ptr: *const clap_audio_port_configuration_request, len: u32) -> Self {
        Self {
            // SAFETY: the caller ensures the pointer is valid, so we can create a slice here.
            raw: unsafe { std::slice::from_raw_parts(ptr, len as usize) },
        }
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of requests in the list.
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns the request at the given index, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<AudioPortsRequest<'a>> {
        // SAFETY: validity is ensured by the lifetime of self.
        self.raw
            .get(index)
            .map(|r| unsafe { AudioPortsRequest::from_raw(*r) })
    }

    /// Returns an iterator over all requests in the list.
    pub fn iter(
        &'a self,
    ) -> impl ExactSizeIterator<Item = AudioPortsRequest<'a>> + DoubleEndedIterator + 'a {
        IntoIterator::into_iter(self)
    }
}

impl<'a> IntoIterator for &'a AudioPortsRequestList<'a> {
    type Item = AudioPortsRequest<'a>;
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, clap_audio_port_configuration_request>,
        fn(&clap_audio_port_configuration_request) -> AudioPortsRequest<'a>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: validity is ensured by the lifetime of self.
        self.raw
            .iter()
            .map(|r| unsafe { AudioPortsRequest::from_raw(*r) })
    }
}

impl Debug for AudioPortsRequestList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a> AudioPortsRequest<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided request is valid for the duration of lifetime `'a`.
    unsafe fn from_raw(raw: clap_audio_port_configuration_request) -> Self {
        // SAFETY: the caller ensures the pointer is valid, so we can dereference it here.
        unsafe {
            Self {
                is_input: raw.is_input,
                port_index: raw.port_index,
                channel_count: raw.channel_count,
                port_info: AudioPortsRequestPort::Other(AudioPortType::from_raw(raw.port_type)),
            }
        }
    }
}

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
