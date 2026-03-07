use crate::configurable_audio_ports::{AudioPortsRequest, PluginConfigurableAudioPorts};
use clack_host::plugin::InactivePluginMainThreadHandle;

impl PluginConfigurableAudioPorts {
    /// Returns true if the given configurations can be applied using [`apply_configuration`](Self::apply_configuration).
    pub fn can_apply_configuration(
        &self,
        plugin: &mut InactivePluginMainThreadHandle,
        requests: &[AudioPortsRequest<'_>],
    ) -> bool {
        let requests_len = match u32::try_from(requests.len()) {
            Ok(len) => len,
            Err(_) => return false, // too many requests to fit in a u32, so we can't apply the configuration
        };

        // SAFETY: This type ensures the function pointer is valid.
        unsafe {
            let requests = AudioPortsRequest::as_raw_slice(requests);
            match plugin.use_extension(&self.0).can_apply_configuration {
                None => false,
                Some(can_apply_configuration) => {
                    can_apply_configuration(plugin.as_raw(), requests.as_ptr(), requests_len)
                }
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
        requests: &[AudioPortsRequest<'_>],
    ) -> bool {
        let requests_len = match u32::try_from(requests.len()) {
            Ok(len) => len,
            Err(_) => return false, // too many requests to fit in a u32, so we can't apply the configuration
        };

        // SAFETY: This type ensures the function pointer is valid.
        unsafe {
            let requests = AudioPortsRequest::as_raw_slice(requests);
            match plugin.use_extension(&self.0).apply_configuration {
                None => false,
                Some(apply_configuration) => {
                    apply_configuration(plugin.as_raw(), requests.as_ptr(), requests_len)
                }
            }
        }
    }
}
