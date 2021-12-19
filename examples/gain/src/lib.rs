use clap_audio_params::{ParamInfo, ParamsDescriptor, PluginParams};
use clap_plugin::extension::ExtensionDeclarations;
use clap_plugin::host::HostHandle;
use clap_plugin::process::audio::Audio;
use clap_plugin::process::Process;
use clap_plugin::{
    entry::{PluginEntry, PluginEntryDescriptor},
    host::HostInfo,
    plugin::{Plugin, PluginDescriptor, PluginInstance},
};

pub struct GainPlugin;

impl<'a> Plugin<'a> for GainPlugin {
    const ID: &'static [u8] = b"gain\0";

    fn new(_host: HostHandle<'a>) -> Option<Self> {
        Some(Self)
    }

    fn process(&self, _process: &Process, mut audio: Audio) {
        // Only handle f32 samples for simplicity
        let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }
    }

    fn declare_extensions(&self, builder: &mut ExtensionDeclarations<Self>) {
        builder.register::<ParamsDescriptor>()
    }
}

// TODO: properly implement and design this API
impl<'a> PluginParams<'a> for GainPlugin {
    fn count(&self) -> u32 {
        0
    }

    fn get_info(&self, _param_index: i32, _info: &mut ParamInfo) -> bool {
        false
    }

    fn get_value(&self, _param_id: u32) -> Option<f64> {
        None
    }

    fn value_to_text(&self, _param_id: u32, _value: f64, _output_buf: &mut [u8]) -> bool {
        false
    }

    fn text_to_value(&self, _param_id: u32, _text: &str) -> Option<f64> {
        None
    }

    fn flush(&self) {}
}

pub struct GainEntry;

impl PluginEntry for GainEntry {
    fn plugin_count() -> u32 {
        1
    }

    fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor> {
        match index {
            0 => Some(GainPlugin::DESCRIPTOR),
            _ => None,
        }
    }

    fn create_plugin<'a>(host_info: HostInfo<'a>, plugin_id: &[u8]) -> Option<PluginInstance<'a>> {
        match plugin_id {
            GainPlugin::ID => Some(PluginInstance::new::<GainPlugin>(host_info)),
            _ => None,
        }
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static clap_plugin_entry: PluginEntryDescriptor = GainEntry::DESCRIPTOR;
