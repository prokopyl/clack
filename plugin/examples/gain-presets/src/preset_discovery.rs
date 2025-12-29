use clack_extensions::preset_discovery::{
    IndexerInfo, PresetDiscoveryFactoryImpl, ProviderDescriptor, ProviderInstance, plugin::Provider,
};
use clack_plugin::host::HostInfo;
use std::ffi::CStr;

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

        ProviderInstance::new(indexer_info, &self.desc, |indexer| GainPresetProvider);
        todo!()
    }
}

pub struct GainPresetProvider;

impl<'a> Provider<'a> for GainPresetProvider {}
