use super::*;
use super::{PluginAsVST3, PluginInfoAsVST3, SupportedNoteExpressions};
use clack_host::plugin::PluginMainThreadHandle;

impl PluginFactoryAsVST3<'_> {
    /// Returns the extra VST3 information for the plugin with the given index, if available.
    #[inline]
    pub fn vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3<'_>> {
        let get_info = self.0.get().get_vst3_info?;

        // SAFETY:
        // - This type guarantees the factory pointer is valid
        // - We got the function from the same instance the pointer points to
        let ptr = unsafe { get_info(self.0.as_ptr(), index) };

        if ptr.is_null() {
            return None;
        }

        // SAFETY: We just checked the pointer is non-null. The rest should be valid as per the CLAP spec
        unsafe { Some(PluginInfoAsVST3::from_raw(ptr)) }
    }
}

impl PluginAsVST3 {
    /// Returns the number of MIDI channels supported by the plugin for a given note port.
    #[inline]
    pub fn get_num_midi_channels(
        &self,
        plugin: &mut PluginMainThreadHandle<'_>,
        note_port: u32,
    ) -> u32 {
        let Some(ext) = plugin.use_extension(&self.0).get_num_midi_channels else {
            return 0;
        };

        // SAFETY: Plugin pointer is valid, as it comes from as_raw
        unsafe { ext(plugin.as_raw(), note_port) }
    }

    /// Returns the supported note expression types for a given note port.
    #[inline]
    pub fn supported_note_expressions(
        &self,
        plugin: &mut PluginMainThreadHandle<'_>,
    ) -> SupportedNoteExpressions {
        let Some(ext) = plugin.use_extension(&self.0).supported_note_expressions else {
            return SupportedNoteExpressions::empty();
        };

        // SAFETY: Plugin pointer is valid, as it comes from as_raw
        let supported = unsafe { ext(plugin.as_raw()) };
        SupportedNoteExpressions::from_bits_truncate(supported)
    }
}
