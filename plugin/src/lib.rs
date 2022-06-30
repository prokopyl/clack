#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

pub mod bundle;
pub mod extensions;
pub mod factory;
pub mod host;
pub mod plugin;
pub mod process;

pub use clack_common::events;
pub use clack_common::ports;
pub use clack_common::stream;

pub mod prelude {
    pub use crate::bundle::{PluginEntry, PluginEntryDescriptor, SinglePluginEntry};
    pub use crate::events::{
        io::{InputEvents, OutputEvents},
        UnknownEvent,
    };
    pub use crate::extensions::PluginExtensions;
    pub use crate::host::{HostAudioThreadHandle, HostHandle, HostMainThreadHandle};
    pub use crate::plugin::{
        AudioConfiguration, Plugin, PluginError, PluginMainThread, PluginShared,
    };
    pub use crate::process::{audio::Audio, events::ProcessEvents, Process, ProcessStatus};
}
