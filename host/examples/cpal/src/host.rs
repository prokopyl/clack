use crate::discovery::FoundBundlePlugin;

use clack_extensions::audio_ports::{HostAudioPortsImpl, PluginAudioPorts, RescanType};
use clack_extensions::gui::{GuiSize, HostGui};
use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_extensions::params::{
    HostParams, HostParamsImplMainThread, HostParamsImplShared, ParamClearFlags, ParamRescanFlags,
};
use clack_extensions::timer::{HostTimer, PluginTimer};
use clack_host::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::ffi::CString;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

/// Audio related routines and utilities.
mod audio;
/// GUI handling.
mod gui;
/// A Timer implementation.
mod timer;

use audio::*;
use clack_extensions::note_ports::{HostNotePortsImpl, NoteDialects, NotePortRescanFlags};
use gui::*;
use timer::*;

/// Messages that can be sent to the main thread from any of the plugin's threads.
enum MainThreadMessage {
    /// Request to run the "on_main_thread" callback.
    RunOnMainThread,
    /// Informs the host that the plugin's floating window has been closed.
    GuiClosed,
    /// The plugin requests that the window it's GUI is embedded in to be resized to a given size.
    GuiRequestResized {
        /// The size of the window the plugin's GUI would like to have.
        new_size: GuiSize,
    },
}

/// Our host implementation.
pub struct CpalHost;

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

/// Contains all the callbacks that the plugin gave us to call.
/// (This is unused in this example, but this is kept here for demonstration purposes)
#[allow(dead_code)]
struct PluginCallbacks<'a> {
    /// A handle to the plugin's Audio Ports extension, if it supports it.
    audio_ports: Option<&'a PluginAudioPorts>,
}

/// Data, accessible by all the plugin's threads.
pub struct CpalHostShared<'a> {
    /// The sender side of the channel to the main thread.
    sender: Sender<MainThreadMessage>,
    /// The plugin callbacks.
    /// This is stored in a separate, thread-safe lock because the initializing method might be
    /// called concurrently with any other thread-safe host methods.
    callbacks: OnceLock<PluginCallbacks<'a>>,
    /// The plugin's shared handle.
    /// This is stored in a separate, thread-safe lock because the instantiation might complete
    /// concurrently with any other thread-safe host methods.
    plugin: OnceLock<InitializedPluginHandle<'a>>,
}

impl<'a> CpalHostShared<'a> {
    /// Initializes the shared data.
    fn new(sender: Sender<MainThreadMessage>) -> Self {
        Self {
            sender,
            callbacks: OnceLock::new(),
            plugin: OnceLock::new(),
        }
    }
}

impl<'a> HostShared<'a> for CpalHostShared<'a> {
    fn initializing(&self, instance: InitializingPluginHandle<'a>) {
        let _ = self.callbacks.set(PluginCallbacks {
            audio_ports: instance.get_extension(),
        });
    }

    fn request_restart(&self) {
        // We don't support restarting plugins
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

/// Data only accessible by the main thread.
pub struct CpalHostMainThread<'a> {
    /// A reference to shared host data.
    /// (this is unused in this example, but this is kept here for demonstration purposes).
    _shared: &'a CpalHostShared<'a>,
    /// A handle to the plugin instance.
    plugin: Option<InitializedPluginHandle<'a>>,

    /// A handle to the plugin's Timer extension, if it supports it.
    /// This is placed here, since only the main thread will ever use that extension.
    timer_support: Option<&'a PluginTimer>,
    /// The timer implementation.
    timers: Timers,
    /// The GUI implementation, if supported.
    gui: Option<Gui<'a>>,
}

impl<'a> CpalHostMainThread<'a> {
    /// Initializes the main thread data.
    fn new(shared: &'a CpalHostShared<'a>) -> Self {
        Self {
            _shared: shared,
            plugin: None,
            timer_support: None,
            timers: Timers::new(),
            gui: None,
        }
    }
}

impl<'a> HostMainThread<'a> for CpalHostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.gui = instance
            .get_extension()
            .map(|gui| Gui::new(gui, &mut instance));

        self.timer_support = instance.get_extension();
        self._shared
            .plugin
            .set(instance)
            .expect("This is the only method that should set the instance handles.");
        self.plugin = Some(instance);
    }
}

/// Runs a given plugin.
///
/// This sets up everything, instantiates the plugin, and creates and connects the audio and MIDI
/// streams.
///
/// If the plugin has a GUI this host supports, this opens its, and keeps the host and streams
/// running until the window is closed.
///
/// Otherwise, the plugin runs headless, and will keep running until the process is killed.
pub fn run(plugin: FoundBundlePlugin) -> Result<(), Box<dyn Error>> {
    let host_info = host_info();
    let plugin_id = CString::new(plugin.plugin.id.as_str())?;
    let (sender, receiver) = unbounded();

    let mut instance = PluginInstance::<CpalHost>::new(
        |_| CpalHostShared::new(sender.clone()),
        |shared| CpalHostMainThread::new(shared),
        &plugin.bundle,
        &plugin_id,
        &host_info,
    )?;

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

/// Runs the UI in a floating-window mode.
///
/// This blocks until the window is closed.
// Note: not very-well tested
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
            MainThreadMessage::GuiClosed { .. } => {
                println!("Window closed!");
                break;
            }
            _ => {}
        }
    }

    instance.main_thread_host_data_mut().destroy_gui();

    Ok(())
}

/// Runs the UI in an embedded-window mode.
///
/// This blocks until the window is closed.
fn run_gui_embedded(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
) -> Result<(), Box<dyn Error>> {
    let main_thread = instance.main_thread_host_data_mut();
    println!("Opening GUI in embedded mode");

    let event_loop = EventLoop::new()?;
    let gui = main_thread.gui.as_mut().unwrap();
    let plugin = main_thread.plugin.as_mut().unwrap();

    let mut window = Some(gui.open_embedded(plugin, &event_loop)?);

    let uses_logical_pixels = gui.configuration.unwrap().api_type.uses_logical_size();

    event_loop.run(move |event, target| {
        while let Ok(message) = receiver.try_recv() {
            match message {
                MainThreadMessage::RunOnMainThread => instance.call_on_main_thread_callback(),
                MainThreadMessage::GuiRequestResized { new_size } => {
                    let new_size: Size = if uses_logical_pixels {
                        LogicalSize {
                            width: new_size.width,
                            height: new_size.height,
                        }
                        .into()
                    } else {
                        PhysicalSize {
                            width: new_size.width,
                            height: new_size.height,
                        }
                        .into()
                    };

                    let _ = window.as_mut().unwrap().request_inner_size(new_size);
                }
                _ => {}
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("Plugin window closed, stopping.");
                    instance.main_thread_host_data_mut().destroy_gui();
                    window.take(); // Drop the window
                    return;
                }
                WindowEvent::Destroyed => {
                    target.exit();
                    return;
                }
                WindowEvent::Resized(size) => {
                    let window = window.as_ref().unwrap();
                    let scale_factor = window.scale_factor();

                    let actual_size = instance
                        .main_thread_host_data_mut()
                        .resize_gui(size, scale_factor);

                    if actual_size != size.into() {
                        let _ = window.request_inner_size(actual_size);
                    }
                }
                _ => {}
            },
            Event::LoopExiting => {
                instance.main_thread_host_data_mut().destroy_gui();
            }
            _ => {}
        }

        let main_thread = instance.main_thread_host_data_mut();
        main_thread.tick_timers();
        let wait_duration = main_thread
            .timers
            .smallest_duration()
            .unwrap_or(Duration::from_millis(60));
        target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + wait_duration));
    })?;

    // Just to let any eventual background thread properly close (looking at you JUCE)
    std::thread::sleep(Duration::from_millis(100));

    Ok(())
}

/// Runs the plugin heedlessly, without an UI event loop.
///
/// This blocks forever, until the process is killed.
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

/// Information about this host.
fn host_info() -> HostInfo {
    HostInfo::new(
        "Clack example CPAL host",
        "Clack",
        "https://github.com/prokopyl/clack",
        "0.0.0",
    )
    .unwrap()
}

impl<'a> HostLogImpl for CpalHostShared<'a> {
    fn log(&self, severity: LogSeverity, message: &str) {
        if severity <= LogSeverity::Debug {
            return;
        };
        // Note: writing to stdout isn't realtime-safe, and should ideally be avoided.
        // This is only "good enough™" for an example.
        // A mpsc ringbuffer with support for dynamically-sized messages (`?Sized`) should be used to
        // send the logs the main thread without allocating or blocking.
        eprintln!("[{severity}] {message}")
    }
}

impl<'a> HostAudioPortsImpl for CpalHostMainThread<'a> {
    fn is_rescan_flag_supported(&self, _flag: RescanType) -> bool {
        false
    }

    fn rescan(&mut self, _flag: RescanType) {
        // We don't support audio ports changing on the fly
    }
}

impl<'a> HostNotePortsImpl for CpalHostMainThread<'a> {
    fn supported_dialects(&self) -> NoteDialects {
        NoteDialects::CLAP
    }

    fn rescan(&mut self, _flags: NotePortRescanFlags) {
        // We don't support note ports changing on the fly
    }
}

impl<'a> HostParamsImplMainThread for CpalHostMainThread<'a> {
    fn rescan(&mut self, _flags: ParamRescanFlags) {
        // We don't track param values at all
    }

    fn clear(&mut self, _param_id: u32, _flags: ParamClearFlags) {}
}

impl<'a> HostParamsImplShared for CpalHostShared<'a> {
    fn request_flush(&self) {
        // Can never flush events when not processing: we're never not processing
    }
}
