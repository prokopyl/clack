#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

use crate::audio::GainPluginAudioProcessor;
use crate::params::GainParamsLocal;
use crate::{gui::GainPluginGui, params::GainParamsShared};
use clack_extensions::{audio_ports::*, gui::PluginGui, params::*, state::PluginState};
use clack_plugin::prelude::*;
use std::sync::Arc;

mod audio;
mod gui;
mod params;

/// The type that represents our plugin in Clack.
///
/// This is what implements the [`Plugin`] trait, where all the other subtypes are attached.
pub struct GainPlugin;

impl Plugin for GainPlugin {
    type AudioProcessor<'a> = GainPluginAudioProcessor<'a>;
    type Shared<'a> = GainPluginShared;
    type MainThread<'a> = GainPluginMainThread<'a>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&GainPluginShared>,
    ) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginParams>()
            .register::<PluginState>()
            .register::<PluginGui>();
    }
}

impl DefaultPluginFactory for GainPlugin {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("org.rust-audio.clack.gain-egui", "Clack Gain EGUI Example")
            .with_features([AUDIO_EFFECT, STEREO])
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(GainPluginShared {
            params: Arc::new(GainParamsShared::new()),
        })
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Self::MainThread {
            shared,
            params: GainParamsLocal::new(&shared.params),
            gui: None,
        })
    }
}

/// The plugin data that gets shared between the Main Thread and the Audio Thread.
pub struct GainPluginShared {
    /// The plugin's parameter values.
    params: Arc<GainParamsShared>,
}

impl PluginShared<'_> for GainPluginShared {}

/// The data that belongs to the main thread of our plugin.
pub struct GainPluginMainThread<'a> {
    /// The local state of the parameters
    params: GainParamsLocal,
    /// A reference to the plugin's shared data.
    shared: &'a GainPluginShared,
    /// The plugin's GUI state and context
    gui: Option<GainPluginGui>,
}

impl<'a> PluginMainThread<'a, GainPluginShared> for GainPluginMainThread<'a> {
    fn on_main_thread(&mut self) {
        if let Some(gui) = &self.gui {
            gui.request_repaint()
        }
    }
}

clack_export_entry!(SinglePluginEntry<GainPlugin>);
