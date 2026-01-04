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
    pub use crate::{
        clack_export_entry,
        entry::{DefaultPluginFactory, Entry, EntryDescriptor, SinglePluginEntry},
        events::{
            Event, EventHeader, Pckn, UnknownEvent,
            io::{InputEvents, OutputEvents},
        },
        extensions::PluginExtensions,
        host::{HostAudioProcessorHandle, HostMainThreadHandle, HostSharedHandle},
        plugin::{
            Plugin, PluginAudioProcessor, PluginDescriptor, PluginError, PluginMainThread,
            PluginShared,
        },
        process::{
            Audio, Events, PluginAudioConfiguration, Process, ProcessStatus,
            audio::{ChannelPair, SampleType},
        },
        utils::ClapId,
    };
}

#[doc = include_str!("../../README.md")]
const _MAIN_README_TEST: () = {};

use std::ptr::null_mut;
use std::alloc::Layout;

#[cfg(target_arch="wasm32")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return null_mut();
    }

    let Ok(layout) = Layout::from_size_align(size, 1) else {
        return null_mut();
    };

    // SAFETY: we just checked above that size is non-zero
    unsafe { std::alloc::alloc_zeroed(layout) }
}
