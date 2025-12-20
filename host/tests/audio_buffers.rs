use clack_host::prelude::*;
use clack_host::process::StartedPluginAudioProcessor;
use clack_host::process::audio_buffers_v2::{AudioPortBuffers, InputAudioPort};
use clack_plugin::prelude::*;
use std::error::Error;

struct MyPlugin;

impl Plugin for MyPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = ();
}

impl DefaultPluginFactory for MyPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("my.plugin", "My plugin")
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread(_host: HostMainThreadHandle, _shared: &()) -> Result<(), PluginError> {
        Ok(())
    }
}

#[test]
pub fn can_process_audio() -> Result<(), Box<dyn Error>> {
    let mut buf = [0.0; 4];
    let mut audio_ports = AudioPortBuffers::new();

    with_audio_processor(|audio_processor| {
        let inputs = audio_ports.with_inputs(&buf);
        // FOO
        Ok(())
    })
}

fn with_audio_processor(
    handler: impl FnOnce(StartedPluginAudioProcessor<()>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let host_info = HostInfo::default();
    let bundle = PluginBundle::load_from_clack::<SinglePluginEntry<MyPlugin>>(c"")?;
    let mut plugin = PluginInstance::<()>::new(|_| (), |_| (), &bundle, c"my.plugin", &host_info)?;
    let audio_processor = plugin
        .activate(
            |_, _| (),
            PluginAudioConfiguration {
                max_frames_count: 4,
                min_frames_count: 4,
                sample_rate: 44_100.0,
            },
        )?
        .start_processing()?;

    handler(audio_processor)
}
