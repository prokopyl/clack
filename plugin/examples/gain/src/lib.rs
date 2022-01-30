#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

use clack_extensions::audio_ports::{
    AudioPortInfoWriter, PluginAudioPortsImplementation, SampleSize,
};
use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfo, PluginParams};
use clack_extensions::state::{PluginState, PluginStateImplementation};

use clack_plugin::{
    events::event_types::NoteEvent,
    plugin::PluginDescriptor,
    prelude::*,
    stream::{InputStream, OutputStream},
};

use clack_plugin::events::event_types::NoteOnEvent;
use clack_plugin::events::Event;
use std::io::Read;

pub struct GainPlugin;

impl<'a> Plugin<'a> for GainPlugin {
    type Shared = ();
    type MainThread = GainPluginMainThread;

    const DESCRIPTOR: &'static PluginDescriptor = &PluginDescriptor::new(b"gain\0");

    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &(),
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self)
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        events: ProcessEvents,
    ) -> Result<ProcessStatus, PluginError> {
        // Only handle f32 samples for simplicity
        let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }

        for e in events.input {
            if let Some(NoteOnEvent(ne)) = e.as_event() {
                events.output.push_back(
                    NoteOnEvent(NoteEvent::new(
                        *ne.header(),
                        ne.port_index(),
                        ne.key(),
                        ne.channel(),
                        ne.velocity() * 2.0,
                    ))
                    .as_unknown(),
                );
            } else {
                events.output.push_back(e)
            }
        }

        //self.flush(events.input, events.output);

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &()) {
        // builder.register::<PluginParams>().register::<PluginState>(); TODO
    }
}
/*
impl<'a> PluginParamsImpl<'a> for GainPlugin {
    fn flush(
        &mut self,
        _input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut EventList,
    ) {
    }
}
*/
pub struct GainPluginMainThread {
    rusting: u32,
}

impl<'a> PluginMainThread<'a, ()> for GainPluginMainThread {
    fn new(_host: HostMainThreadHandle<'a>, _shared: &()) -> Result<Self, PluginError> {
        Ok(Self { rusting: 0 })
    }
}
/*
impl<'a> PluginMainThreadParams<'a> for GainPluginMainThread {
    fn count(&self) -> u32 {
        1
    }

    fn get_info(&self, param_index: u32, info: &mut ParamInfoWriter) {
        if param_index > 0 {
            return;
        }

        info.set(
            ParamInfo::new(0)
                .with_name("Rusting")
                .with_module("gain/rusting")
                .with_default_value(0.0)
                .with_value_bounds(0.0, 1000.0)
                .with_flags(ParamInfoFlags::IS_STEPPED),
        )
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
    ) -> ::core::fmt::Result {
        use ::core::fmt::Write;
        println!("Format param {}, value {}", param_id, value);

        if param_id == 0 {
            write!(writer, "{} crabz", value as u32)
        } else {
            Ok(())
        }
    }

    fn text_to_value(&self, _param_id: u32, _text: &str) -> Option<f64> {
        None
    }

    fn flush(&mut self, input_events: &InputEvents, _output_events: &mut OutputEvents) {
        let value_events = input_events.iter().filter_map(|e| match e.event()? {
            Event::ParamValue(v) => Some(v),
            _ => None,
        });

        for value in value_events {
            if value.param_id() == 0 {
                self.rusting = value.value() as u32;
            }
        }
    }
}

impl PluginStateImplementation for GainPluginMainThread {
    fn load(&mut self, input: &mut InputStream) -> std::result::Result<(), PluginError> {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        let msg = String::from_utf8_lossy(&buf);
        println!("Loaded: {}", msg);

        Ok(())
    }

    fn save(&mut self, input: &mut OutputStream) -> std::result::Result<(), PluginError> {
        use std::io::Write;

        write!(
            input,
            "Hello! We are rusting with {} crabz today",
            self.rusting
        )?;
        Ok(())
    }
}

impl PluginAudioPortsImplementation for GainPluginMainThread {
    #[inline]
    fn count(&self, _is_input: bool) -> usize {
        1
    }

    #[inline]
    fn get(&self, _is_input: bool, index: usize, writer: &mut AudioPortInfoWriter) {
        if index != 0 {
            return;
        }

        writer.set(
            0,
            "main",
            2,
            SampleSize::F32,
            true,
            false,
            true,
        );
    }
}*/

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static clap_plugin_entry: PluginEntryDescriptor = SinglePluginEntry::<GainPlugin>::DESCRIPTOR;
