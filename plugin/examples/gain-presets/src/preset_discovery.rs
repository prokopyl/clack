use crate::GainPluginMainThread;
use clack_extensions::preset_discovery::{
    FileType, Flags, IndexerInfo, Location, LocationData, PluginPresetLoadImpl,
    PresetDiscoveryFactoryImpl, ProviderDescriptor, ProviderInstance, plugin::MetadataReceiver,
    plugin::ProviderImpl,
};
use clack_plugin::plugin::PluginError;
use clack_plugin::utils::UniversalPluginId;
use std::ffi::{CStr, CString};
use std::str::FromStr;

struct Preset {
    name: &'static CStr,
    volume: f32,
}

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

pub struct GainPresetDiscoveryFactory {
    desc: ProviderDescriptor,
}

impl GainPresetDiscoveryFactory {
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
                dbg!(&indexer);

                indexer.declare_filetype(FileType {
                    name: c"Internal",
                    description: None,
                    file_extension: None,
                });
                indexer.declare_location(LocationData {
                    name: c"Default",
                    flags: Flags::IS_FACTORY_CONTENT,
                    location: Location::Plugin,
                });

                GainPresetProvider
            },
        ))
    }
}

pub struct GainPresetProvider;

impl<'a> ProviderImpl<'a> for GainPresetProvider {
    fn get_metadata(&mut self, location: Location, receiver: &mut MetadataReceiver) {
        dbg!(location);
        for (i, preset) in PRESETS.iter().enumerate() {
            let load_key = CString::new(i.to_string()).unwrap();
            receiver.begin_preset(Some(preset.name), Some(&load_key));
            receiver.add_plugin_id(UniversalPluginId::clap(
                c"org.rust-audio.clack.gain-presets",
            ));
            receiver.add_creator(c"Me!");
        }
    }
}

impl PluginPresetLoadImpl for GainPluginMainThread<'_> {
    // TODO: fully support errors?
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
