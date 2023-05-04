#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(unsafe_code)]

use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfoData, PluginParams};
use std::ffi::CStr;

use clack_plugin::{plugin::descriptor::PluginDescriptor, prelude::*};

use clack_extensions::audio_ports::{
    AudioPortFlags, AudioPortInfoData, AudioPortInfoWriter, AudioPortType, PluginAudioPorts,
    PluginAudioPortsImpl,
};
use clack_plugin::plugin::descriptor::StaticPluginDescriptor;
use clack_plugin::process::audio::ChannelPair;
use clack_plugin::utils::Cookie;

pub struct GainPlugin<'a> {
    _host: HostAudioThreadHandle<'a>,
}

impl<'a> Plugin<'a> for GainPlugin<'a> {
    type Shared = GainPluginShared<'a>;
    type MainThread = GainPluginMainThread<'a>;

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"org.rust-audio.clack.gain\0").unwrap(),
            name: CStr::from_bytes_with_nul(b"Clack Gain Example\0").unwrap(),
            features: Some(&[SYNTHESIZER, STEREO]),
            ..Default::default()
        })
    }

    fn activate(
        host: HostAudioThreadHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &'a GainPluginShared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        for channel_pair in audio
            .port_pairs()
            // Filter out any non-f32 data, in case host is misbehaving and sends f64 data
            .filter_map(|mut p| p.channels().ok()?.into_f32())
            .flatten()
        {
            let buf = match channel_pair {
                ChannelPair::InputOnly(_) => continue, // Ignore extra inputs
                ChannelPair::OutputOnly(o) => {
                    // Just set extra outputs to 0
                    o.fill(0.0);
                    continue;
                }
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    o
                }
                ChannelPair::InPlace(o) => o,
            };

            for x in buf {
                *x *= 2.0;
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &GainPluginShared) {
        builder
            .register::<PluginParams>()
            .register::<PluginAudioPorts>();
    }
}

impl<'a> PluginParamsImpl for GainPlugin<'a> {
    fn flush(
        &mut self,
        _input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
    }
}

impl<'a> PluginAudioPortsImpl for GainPluginMainThread<'a> {
    fn count(&self, _is_input: bool) -> u32 {
        1
    }

    fn get(&self, _is_input: bool, index: u32, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfoData {
                id: 0,
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

pub struct GainPluginShared<'a> {
    _host: HostHandle<'a>,
}

impl<'a> PluginShared<'a> for GainPluginShared<'a> {
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }
}

pub struct GainPluginMainThread<'a> {
    rusting: u32,
    #[allow(unused)]
    shared: &'a GainPluginShared<'a>,

    _host: HostMainThreadHandle<'a>,
}

impl<'a> PluginMainThread<'a, GainPluginShared<'a>> for GainPluginMainThread<'a> {
    fn new(
        host: HostMainThreadHandle<'a>,
        shared: &'a GainPluginShared,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            rusting: 0,
            shared,
            _host: host,
        })
    }
}

impl<'a> PluginMainThreadParams for GainPluginMainThread<'a> {
    fn count(&self) -> u32 {
        1
    }

    fn get_info(&self, param_index: u32, info: &mut ParamInfoWriter) {
        if param_index > 0 {
            return;
        }

        info.set(&ParamInfoData {
            id: 0,
            name: "Rusting",
            module: "gain/rusting",
            default_value: 0.0,
            min_value: 0.0,
            max_value: 1000.0,
            flags: ParamInfoFlags::IS_STEPPED,
            cookie: Cookie::empty(),
        })
    }

    fn get_value(&self, param_id: u32) -> Option<f64> {
        if param_id == 0 {
            Some(self.rusting as f64)
        } else {
            None
        }
    }

    fn value_to_text(
        &self,
        param_id: u32,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        use ::core::fmt::Write;
        println!("Format param {param_id}, value {value}");

        if param_id == 0 {
            write!(writer, "{} crabz", value as u32)
        } else {
            Ok(())
        }
    }

    fn text_to_value(&self, _param_id: u32, _text: &str) -> Option<f64> {
        None
    }

    fn flush(&mut self, _input_events: &InputEvents, _output_events: &mut OutputEvents) {
        /*let value_events = input_events.iter().filter_map(|e| match e.as_event()? {
            Event::ParamValue(v) => Some(v),
            _ => None,
        });

        for value in value_events {
            if value.param_id() == 0 {
                self.rusting = value.value() as u32;
            }
        }*/
    }
}

#[allow(non_upper_case_globals)]
#[allow(unsafe_code)]
#[no_mangle]
pub static clap_entry: PluginEntryDescriptor = SinglePluginEntry::<GainPlugin>::DESCRIPTOR;
