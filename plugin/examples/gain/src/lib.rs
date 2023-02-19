#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(unsafe_code)]

use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfoData, PluginParams};
use std::ffi::CStr;
use std::sync::Arc;

use clack_plugin::{plugin::descriptor::PluginDescriptor, prelude::*};

#[cfg(not(miri))]
use clack_extensions::gui::PluginGui;

use clack_extensions::audio_ports::{
    AudioPortFlags, AudioPortInfoData, AudioPortInfoWriter, AudioPortType, PluginAudioPorts,
    PluginAudioPortsImplementation,
};
use clack_plugin::plugin::descriptor::StaticPluginDescriptor;
use clack_plugin::process::audio::channels::AudioBufferType;
use clack_plugin::utils::Cookie;
use std::sync::atomic::{AtomicI32, Ordering};

#[cfg(not(miri))]
mod gui;

pub struct GainPlugin<'a> {
    shared: &'a GainPluginShared<'a>,
    latest_gain_value: i32,
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
        shared: &'a GainPluginShared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            shared,
            latest_gain_value: 0,
            _host: host,
        })
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        _events: ProcessEvents,
    ) -> Result<ProcessStatus, PluginError> {
        let io = if let Some(io) = audio.zip(0, 0) {
            io
        } else {
            return Ok(ProcessStatus::ContinueIfNotQuiet);
        };

        match io {
            AudioBufferType::F32(io) => {
                // Supports safe in_place processing
                for (input, output) in io {
                    output.set(input.get() * 2.0)
                }
            }
            AudioBufferType::F64(io) => {
                // Supports safe in_place processing
                for (input, output) in io {
                    output.set(input.get() * 2.0)
                }
            }
        }

        /*let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }*/

        let new_gain = self.shared.from_ui.gain.load(Ordering::Relaxed);
        if new_gain != self.latest_gain_value {
            println!("New gain value: {new_gain}");
            self.latest_gain_value = new_gain;
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &GainPluginShared) {
        builder
            .register::<PluginParams>()
            .register::<PluginAudioPorts>();

        #[cfg(not(miri))]
        builder.register::<PluginGui>();
    }
}

impl<'a> PluginParamsImpl<'a> for GainPlugin<'a> {
    fn flush(
        &mut self,
        _input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
    }
}

impl<'a> PluginAudioPortsImplementation for GainPluginMainThread<'a> {
    fn count(&self, _is_input: bool) -> u32 {
        1
    }

    fn get(&self, _is_input: bool, index: u32, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfoData {
                id: 0,
                name: "main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: 0,
            });
        }
    }
}

#[derive(Default)]
pub struct UiAtomics {
    gain: AtomicI32, // in dB TODO
}

pub struct GainPluginShared<'a> {
    from_ui: Arc<UiAtomics>,
    _host: HostHandle<'a>,
}

impl<'a> PluginShared<'a> for GainPluginShared<'a> {
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self {
            from_ui: Arc::new(UiAtomics::default()),
            _host: host,
        })
    }
}

pub struct GainPluginMainThread<'a> {
    rusting: u32,
    #[allow(unused)]
    shared: &'a GainPluginShared<'a>,

    #[cfg(not(miri))]
    gui: gui::MainThreadGui,
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
            #[cfg(not(miri))]
            gui: gui::MainThreadGui::default(),
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
        // TODO
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
