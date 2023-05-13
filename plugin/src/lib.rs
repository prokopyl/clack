#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

extern crate core;

pub mod bundle;
pub mod extensions;
pub mod factory;
pub mod host;
pub mod plugin;
pub mod process;

pub use clack_common::events;
pub use clack_common::stream;
pub use clack_common::utils;

pub mod prelude {
    pub use crate::bundle::{PluginEntry, PluginEntryDescriptor, SinglePluginEntry};
    pub use crate::events::{
        io::{InputEvents, OutputEvents},
        UnknownEvent,
    };
    pub use crate::extensions::PluginExtensions;
    pub use crate::host::{HostAudioThreadHandle, HostHandle, HostMainThreadHandle};
    pub use crate::plugin::{
        descriptor::{PluginDescriptor, StaticPluginDescriptor},
        AudioConfiguration, Plugin, PluginError, PluginMainThread, PluginShared,
    };
    pub use crate::process::{
        audio::{ChannelPair, SampleType},
        Audio, Events, Process, ProcessStatus,
    };
}

#[doc = include_str!("../../README.md")]
const _MAIN_README_TEST: () = {};
