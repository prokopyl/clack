#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "audio-ports")]
pub mod audio_ports;
#[cfg(feature = "audio-ports-config")]
pub mod audio_ports_config;
#[cfg(feature = "event-registry")]
pub mod event_registry;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "latency")]
pub mod latency;
#[cfg(feature = "log")]
pub mod log;
#[cfg(feature = "note-name")]
pub mod note_name;
#[cfg(feature = "note-ports")]
pub mod note_ports;
#[cfg(feature = "params")]
pub mod params;
#[cfg(all(unix, feature = "posix-fd"))]
pub mod posix_fd;
#[cfg(feature = "render")]
pub mod render;
#[cfg(feature = "state")]
pub mod state;
#[cfg(feature = "tail")]
pub mod tail;
#[cfg(feature = "thread-check")]
pub mod thread_check;
#[cfg(feature = "thread-pool")]
pub mod thread_pool;
#[cfg(feature = "timer")]
pub mod timer;
#[cfg(feature = "voice-info")]
pub mod voice_info;

pub(crate) mod utils;

#[cfg(test)]
#[doc(hidden)]
pub mod __doc_utils;
pub mod wrappers;
