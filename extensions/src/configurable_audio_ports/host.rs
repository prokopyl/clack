use crate::configurable_audio_ports::{AudioPortsRequest, PluginConfigurableAudioPorts};
use clack_host::plugin::InactivePluginMainThreadHandle;
use clap_sys::ext::configurable_audio_ports::clap_audio_port_configuration_request;
use std::marker::PhantomData;

/// A [`Vec`]-backed buffer to build a list of [`AudioPortsRequest`]s.
#[derive(Default)]
pub struct AudioPortsRequestListBuffer<'a> {
    buffer: Vec<clap_audio_port_configuration_request>,
    phantom: PhantomData<&'a clap_audio_port_configuration_request>,
}

impl<'a> AudioPortsRequestListBuffer<'a> {
    /// Creates a new, empty buffer.
    pub fn new() -> Self {
        Default::default()
    }

    /// Appends a request to the buffer.
    pub fn push(&mut self, request: AudioPortsRequest<'a>) {
        self.buffer.push(request.as_raw());
    }

    /// Clears all requests from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl<'a> FromIterator<AudioPortsRequest<'a>> for AudioPortsRequestListBuffer<'a> {
    fn from_iter<T: IntoIterator<Item = AudioPortsRequest<'a>>>(iter: T) -> Self {
        Self {
            buffer: iter.into_iter().map(|r| r.as_raw()).collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a> Extend<AudioPortsRequest<'a>> for AudioPortsRequestListBuffer<'a> {
    fn extend<T: IntoIterator<Item = AudioPortsRequest<'a>>>(&mut self, iter: T) {
        self.buffer.extend(iter.into_iter().map(|r| r.as_raw()));
    }
}

impl<'a> AudioPortsRequest<'a> {
    fn as_raw(&self) -> clap_audio_port_configuration_request {
        clap_audio_port_configuration_request {
            is_input: self.is_input,
            port_index: self.port_index,
            channel_count: self.channel_count,
            port_type: self
                .port_info
                .port_type()
                .map(|t| t.0.as_ptr())
                .unwrap_or(std::ptr::null()),
            port_details: std::ptr::null(),
        }
    }
}

impl PluginConfigurableAudioPorts {
    /// Returns true if the given configurations can be applied using [`apply_configuration`](Self::apply_configuration).
    pub fn can_apply_configuration(
        &self,
        plugin: &mut InactivePluginMainThreadHandle,
        list: &AudioPortsRequestListBuffer<'_>,
    ) -> bool {
        // SAFETY: This type ensures the function pointer is valid.
        unsafe {
            match plugin.use_extension(&self.0).can_apply_configuration {
                None => false,
                Some(can_apply_configuration) => can_apply_configuration(
                    plugin.as_raw(),
                    list.buffer.as_ptr(),
                    list.buffer.len() as u32,
                ),
            }
        }
    }

    /// Submit a bunch of configuration requests which will atomically be applied together,
    /// or discarded together.
    ///
    /// Once the configuration is successfully applied, it isn't necessary for the plugin to call
    /// [`HostAudioPorts::rescan`](crate::audio_ports::HostAudioPorts::rescan); and it isn't necessary for the host to scan the
    /// audio ports.
    ///
    /// Returns true if applied, false otherwise.
    pub fn apply_configuration(
        &self,
        plugin: &mut InactivePluginMainThreadHandle,
        list: &AudioPortsRequestListBuffer<'_>,
    ) -> bool {
        // SAFETY: This type ensures the function pointer is valid.
        unsafe {
            match plugin.use_extension(&self.0).apply_configuration {
                None => false,
                Some(apply_configuration) => apply_configuration(
                    plugin.as_raw(),
                    list.buffer.as_ptr(),
                    list.buffer.len() as u32,
                ),
            }
        }
    }
}
