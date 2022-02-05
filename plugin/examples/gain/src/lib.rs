#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{implementation::*, info::ParamInfoData, PluginParams};
use std::sync::Arc;

use clack_plugin::{plugin::PluginDescriptor, prelude::*};

use baseview::WindowHandle;
use clack_extensions::gui::attached::PluginGuiX11;
use clack_extensions::gui::PluginGui;

use std::sync::atomic::{AtomicI32, Ordering};

mod gui;

pub struct GainPlugin<'a> {
    shared: &'a GainPluginShared,
    latest_gain_value: i32,
}

impl<'a> Plugin<'a> for GainPlugin<'a> {
    type Shared = GainPluginShared;
    type MainThread = GainPluginMainThread<'a>;

    const DESCRIPTOR: &'static PluginDescriptor = &PluginDescriptor::new(b"gain\0");

    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        shared: &'a GainPluginShared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            shared,
            latest_gain_value: 0,
        })
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        _events: ProcessEvents,
    ) -> Result<ProcessStatus, PluginError> {
        // Only handle f32 samples for simplicity
        let io = audio.zip(0, 0).unwrap().into_f32().unwrap();

        // Supports safe in_place processing
        for (input, output) in io {
            output.set(input.get() * 2.0)
        }

        let new_gain = self.shared.from_ui.gain.load(Ordering::Relaxed);
        if new_gain != self.latest_gain_value {
            println!("New gain value: {}", new_gain);
            self.latest_gain_value = new_gain;
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &GainPluginShared) {
        builder
            .register::<PluginParams>()
            .register::<PluginGui>()
            .register::<PluginGuiX11>();
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

#[derive(Default)]
pub struct UiAtomics {
    gain: AtomicI32, // in dB TODO
}

pub struct GainPluginShared {
    from_ui: Arc<UiAtomics>,
}

impl<'a> PluginShared<'a> for GainPluginShared {
    fn new(_host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self {
            from_ui: Arc::new(UiAtomics::default()),
        })
    }
}

pub struct GainPluginMainThread<'a> {
    rusting: u32,
    shared: &'a GainPluginShared,

    open_window: Option<WindowHandle>,
}

impl<'a> PluginMainThread<'a, GainPluginShared> for GainPluginMainThread<'a> {
    fn new(
        _host: HostMainThreadHandle<'a>,
        shared: &'a GainPluginShared,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            rusting: 0,
            shared,
            open_window: None,
        })
    }
}

impl<'a> PluginMainThreadParams<'a> for GainPluginMainThread<'a> {
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
            cookie: ::core::ptr::null_mut(),
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
#[no_mangle]
pub static clap_entry: PluginEntryDescriptor = SinglePluginEntry::<GainPlugin>::DESCRIPTOR;
