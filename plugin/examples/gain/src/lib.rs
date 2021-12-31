#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfo, PluginParams};
use clack_plugin::events::event_types::NoteEvent;
use clack_plugin::events::list::EventList;
use clack_plugin::events::{Event, EventType};
use clack_plugin::extension::ExtensionDeclarations;
use clack_plugin::host::{HostHandle, HostInfo};
use clack_plugin::plugin::{PluginMainThread, SampleConfig};
use clack_plugin::process::audio::Audio;
use clack_plugin::process::events::ProcessEvents;
use clack_plugin::process::Process;
use clack_plugin::process::ProcessStatus;
use clack_plugin::{
    entry::{PluginEntry, PluginEntryDescriptor},
    plugin::{Plugin, PluginDescriptor, PluginInstance, Result},
};

pub struct GainPlugin;

impl<'a> Plugin<'a> for GainPlugin {
    type Shared = ();
    type MainThread = GainPluginMainThread;

    const ID: &'static [u8] = b"gain\0";

    fn new(
        _host: HostHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &(),
        _sample_config: SampleConfig,
    ) -> Result<Self> {
        Ok(Self)
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        events: ProcessEvents,
    ) -> Result<ProcessStatus> {
        // Only handle f32 samples for simplicity
        let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }

        events
            .output
            .extend(events.input.iter().map(|e| match e.event() {
                Some(EventType::NoteOn(ne)) => Event::new(
                    e.time(),
                    EventType::NoteOn(NoteEvent::new(
                        ne.port_index(),
                        ne.key(),
                        ne.channel(),
                        ne.velocity() * 2.0,
                    )),
                ),
                _ => *e,
            }));

        self.flush(events.input, events.output);

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut ExtensionDeclarations<Self>, _shared: &()) {
        builder.register::<PluginParams>();
    }
}

impl<'a> PluginParamsImpl<'a> for GainPlugin {
    fn flush(
        &mut self,
        _input_parameter_changes: &EventList,
        _output_parameter_changes: &EventList,
    ) {
    }
}

pub struct GainPluginMainThread {
    rusting: u32,
}

impl<'a> PluginMainThread<'a, ()> for GainPluginMainThread {
    fn new(_host: HostHandle<'a>, _shared: &()) -> Result<Self> {
        Ok(Self { rusting: 0 })
    }
}

impl<'a> PluginMainThreadParams<'a> for GainPluginMainThread {
    fn count(&self) -> u32 {
        1
    }

    fn get_info(&self, param_index: i32, info: &mut ParamInfoWriter) {
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

    fn flush(&mut self, input_events: &EventList, _output_events: &EventList) {
        let value_events = input_events.iter().filter_map(|e| match e.event()? {
            EventType::ParamValue(v) => Some(v),
            _ => None,
        });

        for value in value_events {
            if value.param_id() == 0 {
                self.rusting = value.value() as u32;
            }
        }
    }
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
