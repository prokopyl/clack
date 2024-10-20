use crate::discovery::FoundBundlePlugin;

use clack_extensions::audio_ports::{HostAudioPortsImpl, PluginAudioPorts, RescanType};
use clack_extensions::gui::{GuiSize, HostGui, PluginGui};
use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_extensions::params::{
    HostParams, HostParamsImplMainThread, HostParamsImplShared, ParamClearFlags, ParamRescanFlags,
};
use clack_extensions::timer::{HostTimer, PluginTimer};
use clack_host::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::ffi::CString;
use std::rc::Rc;
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

impl HostHandlers for CpalHost {
    type Shared<'a> = CpalHostShared;
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
struct PluginCallbacks {
    /// A handle to the plugin's Audio Ports extension, if it supports it.
    audio_ports: Option<PluginAudioPorts>,
}

/// Data, accessible by all the plugin's threads.
pub struct CpalHostShared {
    /// The sender side of the channel to the main thread.
    sender: Sender<MainThreadMessage>,
    /// The plugin callbacks.
    /// This is stored in a separate, thread-safe lock because the initializing method might be
    /// called concurrently with any other thread-safe host methods.
    callbacks: OnceLock<PluginCallbacks>,
}

impl CpalHostShared {
    /// Initializes the shared data.
    fn new(sender: Sender<MainThreadMessage>) -> Self {
        Self {
            sender,
            callbacks: OnceLock::new(),
        }
    }
}

impl<'a> SharedHandler<'a> for CpalHostShared {
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
    _shared: &'a CpalHostShared,
    /// A handle to the plugin instance.
    plugin: Option<InitializedPluginHandle<'a>>,

    /// A handle to the plugin's Timer extension, if it supports it.
    /// This is placed here, since only the main thread will ever use that extension.
    timer_support: Option<PluginTimer>,
    /// The timer implementation.
    timers: Rc<Timers>,
    /// A handle to the plugin's GUI extension, if it supports it.
    gui: Option<PluginGui>,
}

impl<'a> CpalHostMainThread<'a> {
    /// Initializes the main thread data.
    fn new(shared: &'a CpalHostShared) -> Self {
        Self {
            _shared: shared,
            plugin: None,
            timer_support: None,
            gui: None,
            timers: Rc::new(Timers::new()),
        }
    }
}

impl<'a> MainThreadHandler<'a> for CpalHostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.gui = instance.get_extension();
        self.timer_support = instance.get_extension();

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

    let _stream = activate_to_stream(&mut instance)?;

    let gui = instance
        .access_handler(|h| h.gui)
        .map(|gui| Gui::new(gui, &mut instance.plugin_handle()));

    let gui = gui.and_then(|gui| Some((gui.needs_floating()?, gui)));

    let Some((needs_floating, gui)) = gui else {
        return run_cli(instance, receiver);
    };

    if needs_floating {
        run_gui_floating(instance, receiver, gui)
    } else {
        run_gui_embedded(instance, receiver, gui)
    }
}

/// Runs the UI in a floating-window mode.
///
/// This blocks until the window is closed.
// Note: not very-well tested
fn run_gui_floating(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
    mut gui: Gui,
) -> Result<(), Box<dyn Error>> {
    println!("Opening GUI in floating mode");
    gui.open_floating(&mut instance.plugin_handle())?;

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

    gui.destroy(&mut instance.plugin_handle());

    Ok(())
}

/// Runs the UI in an embedded-window mode.
///
/// This blocks until the window is closed.
fn run_gui_embedded(
    mut instance: PluginInstance<CpalHost>,
    receiver: Receiver<MainThreadMessage>,
    mut gui: Gui,
) -> Result<(), Box<dyn Error>> {
    println!("Opening GUI in embedded mode");

    let event_loop = EventLoop::new()?;

    let mut window = Some(gui.open_embedded(&mut instance.plugin_handle(), &event_loop)?);

    let uses_logical_pixels = gui.configuration.unwrap().api_type.uses_logical_size();

    let timers = instance.access_handler(|h| h.timer_support.map(|ext| (h.timers.clone(), ext)));

    #[allow(deprecated)]
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
                    gui.destroy(&mut instance.plugin_handle());
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

                    let actual_size = gui.resize(&mut instance.plugin_handle(), size, scale_factor);

                    if actual_size != size.into() {
                        let _ = window.request_inner_size(actual_size);
                    }
                }
                _ => {}
            },
            Event::LoopExiting => {
                gui.destroy(&mut instance.plugin_handle());
            }
            _ => {}
        }

        let wait_duration = if let Some((timers, timer_ext)) = &timers {
            timers.tick_timers(timer_ext, &mut instance.plugin_handle());

            timers
                .smallest_duration()
                .unwrap_or(Duration::from_millis(60))
        } else {
            Duration::from_millis(60)
        };

        target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + wait_duration));
    })?;

    // Just to let any eventual background thread properly close (looking at you JUCE)
    std::thread::sleep(Duration::from_millis(100));

    Ok(())
}

/// Runs the plugin headlessly, without a UI event loop.
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

impl HostLogImpl for CpalHostShared {
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

impl HostAudioPortsImpl for CpalHostMainThread<'_> {
    fn is_rescan_flag_supported(&self, _flag: RescanType) -> bool {
        false
    }

    fn rescan(&mut self, _flag: RescanType) {
        // We don't support audio ports changing on the fly
    }
}

impl HostNotePortsImpl for CpalHostMainThread<'_> {
    fn supported_dialects(&self) -> NoteDialects {
        NoteDialects::CLAP
    }

    fn rescan(&mut self, _flags: NotePortRescanFlags) {
        // We don't support note ports changing on the fly
    }
}

impl HostParamsImplMainThread for CpalHostMainThread<'_> {
    fn rescan(&mut self, _flags: ParamRescanFlags) {
        // We don't track param values at all
    }

    fn clear(&mut self, _param_id: ClapId, _flags: ParamClearFlags) {}
}

impl HostParamsImplShared for CpalHostShared {
    fn request_flush(&self) {
        // Can never flush events when not processing: we're never not processing
    }
}
