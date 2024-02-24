#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

extern crate core;

#[macro_use]
pub mod entry;
pub mod extensions;
pub mod factory;
pub mod host;
pub mod plugin;
pub mod process;

pub(crate) mod internal_utils;

pub use clack_common::events;
pub use clack_common::stream;
pub use clack_common::utils;

/// A helpful prelude re-exporting all the types related to plugin implementation.
pub mod prelude {
    pub use crate::clack_export_entry;
    pub use crate::entry::{DefaultPluginFactory, Entry, EntryDescriptor, SinglePluginEntry};
    pub use crate::events::{
        io::{InputEvents, OutputEvents},
        UnknownEvent,
    };
    pub use crate::extensions::PluginExtensions;
    pub use crate::host::{HostAudioThreadHandle, HostHandle, HostMainThreadHandle};
    pub use crate::plugin::{
        AudioConfiguration, Plugin, PluginAudioProcessor, PluginDescriptor, PluginError,
        PluginMainThread, PluginShared,
    };
    pub use crate::process::{
        audio::{ChannelPair, SampleType},
        Audio, Events, Process, ProcessStatus,
    };
}

#[doc = include_str!("../../README.md")]
const _MAIN_README_TEST: () = {};
