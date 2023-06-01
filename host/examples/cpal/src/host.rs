use crate::host::gui::Gui;
use crate::host::timer::Timers;
use crate::stream::activate_to_stream;
use clack_extensions::audio_ports::{
    AudioPortInfoBuffer, HostAudioPortsImpl, PluginAudioPorts, RescanType,
};
use clack_extensions::audio_ports_config::PluginAudioPortsConfig;
use clack_extensions::gui::{GuiError, GuiSize, HostGui, HostGuiImpl, PluginGui};
use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_extensions::params::{
    HostParams, HostParamsImplMainThread, HostParamsImplShared, ParamClearFlags, ParamRescanFlags,
};
use clack_extensions::timer::{HostTimer, HostTimerImpl, PluginTimer, TimerError, TimerId};
use clack_host::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::ffi::CString;
use std::path::Path;
use std::time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

mod gui;
mod timer;

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
    GuiClosed { was_destroyed: bool },
    WindowClosing,
    Tick,
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
    plugin: Option<PluginMainThreadHandle<'a>>,
    timer_support: Option<&'a PluginTimer>,
    timers: Timers,
    gui: Option<Gui<'a>>,
}

impl<'a> CpalHostMainThread<'a> {
    fn new(shared: &'a CpalHostShared) -> Self {
        Self {
            shared,
            plugin: None,
            timer_support: None,
            timers: Timers::new(),
            gui: None,
        }
    }

    fn tick_timers(&mut self) {
        let Some(timer) = self.timer_support else { return };
        let plugin = self.plugin.as_mut().unwrap();

        for triggered in self.timers.tick_all() {
            timer.on_timer(plugin, triggered);
        }
    }

    fn destroy_gui(&mut self) {
        self.gui
            .as_mut()
            .unwrap()
            .destroy(self.plugin.as_mut().unwrap())
    }
}

impl<'a> HostMainThread<'a> for CpalHostMainThread<'a> {
    fn instantiated(&mut self, mut instance: PluginMainThreadHandle<'a>) {
        self.gui = instance
            .shared()
            .get_extension()
            .map(|gui| Gui::new(gui, &mut instance));

        self.timer_support = instance.shared().get_extension();
        self.plugin = Some(instance);
    }
}

impl<'a> HostTimerImpl for CpalHostMainThread<'a> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, TimerError> {
        Ok(self.timers.register_new(period_ms))
    }

    fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), TimerError> {
        if self.timers.unregister(timer_id) {
            Ok(())
        } else {
            Err(TimerError::UnregisterError)
        }
    }
}

impl<'a> HostParamsImplMainThread for CpalHostMainThread<'a> {
    fn rescan(&mut self, flags: ParamRescanFlags) {
        // todo!()
    }

    fn clear(&mut self, param_id: u32, flags: ParamClearFlags) {
        todo!()
    }
}

impl<'a> HostParamsImplShared for CpalHostShared<'a> {
    fn request_flush(&self) {
        todo!()
    }
}

impl<'a> HostGuiImpl for CpalHostShared<'a> {
    fn resize_hints_changed(&self) {
        // todo!()
    }

    fn request_resize(&self, _new_size: GuiSize) -> Result<(), GuiError> {
        todo!()
    }

    fn request_show(&self) -> Result<(), GuiError> {
        todo!()
    }

    fn request_hide(&self) -> Result<(), GuiError> {
        todo!()
    }

    fn closed(&self, was_destroyed: bool) {
        self.sender
            .send(MainThreadMessage::GuiClosed { was_destroyed })
            .unwrap();
    }
}

impl Host for CpalHost {
    type Shared<'a> = CpalHostShared<'a>;
    type MainThread<'a> = CpalHostMainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder
            .register::<HostLog>()
            .register::<HostGui>()
            .register::<HostTimer>()
            .register::<HostParams>();
    }
}

pub fn run(bundle_path: &Path, plugin_id: &str) -> Result<(), Box<dyn Error>> {
    let bundle = PluginBundle::load(bundle_path)?;

    let host_info = host_info();
    let plugin_id = CString::new(plugin_id)?;
    let (sender, receiver) = unbounded();

    let mut instance = PluginInstance::<CpalHost>::new(
        |_| CpalHostShared::new(sender.clone()),
        |shared| CpalHostMainThread::new(shared),
        &bundle,
        &plugin_id,
        &host_info,
    )?;

    AudioPortsConfig::from_plugin(
        instance.main_thread_host_data().plugin.as_ref().unwrap(),
        instance.shared_host_data().audio_ports,
    );

    let run_ui = match instance
        .main_thread_host_data()
        .gui
        .as_ref()
        .and_then(|g| g.needs_floating())
    {
        Some(true) => run_gui_floating,
        Some(false) => run_gui_embedded,
        None => run_cli,
    };

    let _stream = activate_to_stream(&mut instance)?;

    run_ui(instance, receiver)?;

    Ok(())
}

// TODO: not properly tested
fn run_gui_floating(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
) -> Result<(), Box<dyn Error>> {
    let main_thread = instance.main_thread_host_data_mut();
    println!("Opening GUI in floating mode");
    let gui = main_thread.gui.as_mut().unwrap();
    let plugin = main_thread.plugin.as_mut().unwrap();

    gui.open_floating(plugin)?;

    for message in receiver {
        match message {
            MainThreadMessage::RunOnMainThread => instance.call_on_main_thread_callback(),
            MainThreadMessage::Tick => instance.main_thread_host_data_mut().tick_timers(),
            MainThreadMessage::GuiClosed { was_destroyed } => {
                println!("Window closed!");
                break;
            }
            _ => {}
        }
    }

    instance.main_thread_host_data_mut().destroy_gui();

    Ok(())
}

fn run_gui_embedded(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
) -> Result<(), Box<dyn Error>> {
    let main_thread = instance.main_thread_host_data_mut();
    println!("Opening GUI in embedded mode");

    let event_loop = EventLoop::new();
    let gui = main_thread.gui.as_mut().unwrap();
    let plugin = main_thread.plugin.as_mut().unwrap();

    let mut window = Some(gui.open_embedded(plugin, &event_loop)?);

    event_loop.run(move |event, _target, control_flow| {
        while let Ok(message) = receiver.try_recv() {
            match message {
                MainThreadMessage::RunOnMainThread => instance.call_on_main_thread_callback(),
                MainThreadMessage::WindowClosing => {
                    println!("Window closed!");
                    break;
                }
                _ => {}
            }
        }

        match event {
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    println!("Received close {window_id:?}");
                    instance.main_thread_host_data_mut().destroy_gui();
                    window.take(); // Drop the window
                    control_flow.set_exit();
                    return;
                }
                _ => {}
            },
            Event::LoopDestroyed => {
                instance.main_thread_host_data_mut().destroy_gui();
            }
            _ => {}
        }

        let main_thread = instance.main_thread_host_data_mut();
        main_thread.tick_timers();
        control_flow.set_wait_timeout(
            main_thread
                .timers
                .smallest_duration()
                .unwrap_or(Duration::from_millis(60)),
        );
    });
}

fn run_cli(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
) -> Result<(), Box<dyn Error>> {
    println!("Running headless. Press Ctrl+C to stop processing.");

    for message in receiver {
        if let MainThreadMessage::RunOnMainThread = message {
            instance.call_on_main_thread_callback()
        }
    }

    Ok(())
}

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
        let count = ports.count(handle, true);

        for i in 0..count {
            let config = ports.get(handle, i, true, &mut buf).unwrap();
            println!("config: {config:?}");
        }
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
