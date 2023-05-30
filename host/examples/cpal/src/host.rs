use crate::buffers::CpalAudioOutputBuffers;
use crate::stream::activate_to_stream;
use clack_extensions::audio_ports::{
    AudioPortInfoBuffer, HostAudioPorts, HostAudioPortsImpl, PluginAudioPorts, RescanType,
};
use clack_extensions::audio_ports_config::{AudioPortsConfigBuffer, PluginAudioPortsConfig};
use clack_extensions::gui::PluginGui;
use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_host::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use crossbeam_channel::{unbounded, Sender};
use std::error::Error;
use std::ffi::CString;
use std::path::Path;

pub struct CpalHost;
pub struct CpalHostShared<'a> {
    sender: Sender<MainThreadMessage>,
    plugin: Option<PluginSharedHandle<'a>>,
    gui: Option<&'a PluginGui>,
    audio_ports: Option<&'a PluginAudioPorts>,
    audio_ports_config: Option<&'a PluginAudioPortsConfig>,
}

impl<'a> CpalHostShared<'a> {
    fn new(sender: Sender<MainThreadMessage>) -> Self {
        Self {
            sender,
            plugin: None,
            gui: None,
            audio_ports: None,
            audio_ports_config: None,
        }
    }
}

impl<'a> HostLogImpl for CpalHostShared<'a> {
    fn log(&self, severity: LogSeverity, message: &str) {
        if severity.to_raw() <= LogSeverity::Debug.to_raw() {
            return;
        };
        eprintln!("[{severity}] {message}")
    }
}

impl<'a> HostAudioPortsImpl for CpalHostMainThread<'a> {
    fn is_rescan_flag_supported(&self, flag: RescanType) -> bool {
        true
    }

    fn rescan(&mut self, flag: RescanType) {
        todo!()
    }
}

enum MainThreadMessage {
    RunOnMainThread,
}

impl<'a> HostShared<'a> for CpalHostShared<'a> {
    fn instantiated(&mut self, instance: PluginSharedHandle<'a>) {
        self.gui = instance.get_extension();
        self.audio_ports = instance.get_extension();
        self.plugin = Some(instance);
    }

    fn request_restart(&self) {
        todo!()
    }

    fn request_process(&self) {
        // We never pause, and CPAL is in full control anyway
    }

    fn request_callback(&self) {
        self.sender
            .send(MainThreadMessage::RunOnMainThread)
            .unwrap();
    }
}

pub struct CpalHostMainThread<'a> {
    shared: &'a CpalHostShared<'a>,
    pub plugin: Option<PluginMainThreadHandle<'a>>,
}

impl<'a> HostMainThread<'a> for CpalHostMainThread<'a> {
    fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) {
        self.plugin = Some(instance);
    }
}

impl Host for CpalHost {
    type Shared<'a> = CpalHostShared<'a>;
    type MainThread<'a> = CpalHostMainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder.register::<HostLog>();
    }
}

pub fn run(bundle_path: &Path, plugin_id: &str) -> Result<(), Box<dyn Error>> {
    let bundle = PluginBundle::load(bundle_path)?;

    let host_info = host_info();
    let plugin_id = CString::new(plugin_id)?;
    let (sender, receiver) = unbounded();

    let mut instance = PluginInstance::<CpalHost>::new(
        |_| CpalHostShared::new(sender),
        |shared| CpalHostMainThread {
            shared,
            plugin: None,
        },
        &bundle,
        &plugin_id,
        &host_info,
    )?;

    AudioPortsConfig::from_plugin(
        instance.main_thread_host_data().plugin.as_ref().unwrap(),
        instance.shared_host_data().audio_ports,
    );

    let _stream = activate_to_stream(&mut instance)?;

    for message in receiver {
        match message {
            MainThreadMessage::RunOnMainThread => instance.call_on_main_thread_callback(),
        }
    }
    Ok(())
}

//fn process(audio_processor: StartedPluginAudioProcessor<CpalHost>, data) {

//}

fn host_info() -> HostInfo {
    HostInfo::new(
        "Clack example CPAL host",
        "Clack",
        "https://github.com/prokopyl/clack",
        "0.0.0",
    )
    .unwrap()
}

struct AudioPortsConfig {
    input_channel_counts: Vec<usize>,
    output_channel_counts: Vec<usize>,
}

impl AudioPortsConfig {
    fn from_plugin(handle: &PluginMainThreadHandle, ports: Option<&PluginAudioPorts>) -> Self {
        println!("Scanning plugin ports:");
        let Some(ports) = ports else {
            println!("No ports extension available: assuming single stereo port for input and output");
            return Self {
                input_channel_counts: vec![2],
                output_channel_counts: vec![2],
            }
        };

        let input_channel_counts = vec![];
        let mut buf = AudioPortInfoBuffer::new();
        let count = ports.count(handle, false);
        for i in 0..count {
            let config = ports.get(handle, i, false, &mut buf).unwrap();
            println!("config: {config:?}");
        }

        Self {
            input_channel_counts,
            output_channel_counts: vec![],
        }
    }
}
