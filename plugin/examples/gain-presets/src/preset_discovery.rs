//! Contains all types and implementations related to preset management.

use crate::GainPluginMainThread;
use clack_extensions::preset_discovery::prelude::*;
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::UniversalPluginId;
use std::ffi::{CStr, CString};
use std::str::FromStr;

/// Holds information about a preset for this plugin.
struct Preset {
    /// The name of this preset, displayed to the user
    name: &'static CStr,
    /// The value of the volume parameter this preset sets
    volume: f32,
}

/// A list of all of our built-in presets.
///
/// This could be a more involved data type or stored in an external file, this is done here with
/// an array only for simplicity.
const PRESETS: &[Preset] = &[
    Preset {
        name: c"Unity",
        volume: 1.0,
    },
    Preset {
        name: c"Quieter",
        volume: 0.5,
    },
];

/// Our preset discovery factory implementation
pub struct GainPresetDiscoveryFactory {
    /// The descriptor of our discovery provider
    desc: ProviderDescriptor,
}

impl GainPresetDiscoveryFactory {
    /// Initializes this factory.
    pub fn new() -> Self {
        GainPresetDiscoveryFactory {
            desc: ProviderDescriptor::new(
                "org.rust-audio.clack.gain-presets-provider",
                "Provider for the 'Presets Gain' plugin",
            ),
        }
    }
}

impl PresetDiscoveryFactoryImpl for GainPresetDiscoveryFactory {
    fn provider_count(&self) -> u32 {
        1
    }

    fn provider_descriptor(&self, index: u32) -> Option<&ProviderDescriptor> {
        if index == 0 { Some(&self.desc) } else { None }
    }

    fn create_provider<'a>(
        &'a self,
        indexer_info: IndexerInfo<'a>,
        provider_id: &CStr,
    ) -> Option<ProviderInstance<'a>> {
        if provider_id != self.desc.id().unwrap() {
            return None;
        }

        Some(ProviderInstance::new(
            indexer_info,
            &self.desc,
            |mut indexer| {
                indexer.declare_location(LocationInfo {
                    name: c"Default",
                    flags: Flags::IS_FACTORY_CONTENT,
                    location: Location::Plugin,
                })?;

                Ok(GainPresetProvider)
            },
        ))
    }
}

/// Our preset provider implementation.
///
/// This is pretty much stateless (as it only relies on the PRESETS array), so this has no fields.
pub struct GainPresetProvider;

impl<'a> ProviderImpl<'a> for GainPresetProvider {
    fn get_metadata(
        &mut self,
        location: Location,
        receiver: &mut MetadataReceiver,
    ) -> Result<(), PluginError> {
        if let Location::File { .. } = location {
            // Our presets are only stored in the plugin binary itself (the PRESETS array). The host
            // should not call this with a file location.
            return Err(PluginError::Message(
                "This plugin does not provide presets from files",
            ));
        }

        for (i, preset) in PRESETS.iter().enumerate() {
            let load_key = CString::new(i.to_string())?;
            receiver
                .begin_preset(Some(preset.name), Some(&load_key))?
                .add_plugin_id(UniversalPluginId::clap(
                    c"org.rust-audio.clack.gain-presets",
                ))
                .add_creator(c"Me!");
        }

        Ok(())
    }
}

impl PluginPresetLoadImpl for GainPluginMainThread<'_> {
    fn load_from_location(
        &mut self,
        location: Location,
        load_key: Option<&CStr>,
    ) -> Result<(), PluginError> {
        let Location::Plugin = location else {
            return Err(PluginError::Message("Unsupported plugin location"));
        };

        let Some(load_key) = load_key else {
            return Err(PluginError::Message("Missing load key"));
        };

        let Ok(load_key) = load_key.to_str() else {
            return Err(PluginError::Message("Invalid load key"));
        };

        let Ok(load_key) = usize::from_str(load_key) else {
            return Err(PluginError::Message("Invalid load key"));
        };

        let Some(preset) = PRESETS.get(load_key) else {
            return Err(PluginError::Message("Invalid load key"));
        };

        self.shared.params.set_volume(preset.volume);

        Ok(())
    }
}
