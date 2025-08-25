#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "ambisonic")]
pub mod ambisonic;
#[cfg(feature = "audio-ports")]
pub mod audio_ports;
#[cfg(feature = "audio-ports-activation")]
pub mod audio_ports_activation;
#[cfg(feature = "audio-ports-config")]
pub mod audio_ports_config;
#[cfg(feature = "configurable-audio-ports")]
pub mod configurable_audio_ports;
#[cfg(feature = "context-menu")]
pub mod context_menu;
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
#[cfg(feature = "param-indication")]
pub mod param_indication;
#[cfg(feature = "params")]
pub mod params;
#[cfg(all(unix, feature = "posix-fd"))]
pub mod posix_fd;
#[cfg(feature = "preset-load")]
pub mod preset_load;
#[cfg(feature = "remote-controls")]
pub mod remote_controls;
#[cfg(feature = "render")]
pub mod render;
#[cfg(feature = "state")]
pub mod state;
#[cfg(feature = "state-context")]
pub mod state_context;
#[cfg(feature = "surround")]
pub mod surround;
#[cfg(feature = "tail")]
pub mod tail;
#[cfg(feature = "thread-check")]
pub mod thread_check;
#[cfg(feature = "thread-pool")]
pub mod thread_pool;
#[cfg(feature = "timer")]
pub mod timer;
#[cfg(feature = "track-info")]
pub mod track_info;
#[cfg(feature = "voice-info")]
pub mod voice_info;

pub(crate) mod utils;

#[cfg(test)]
#[doc(hidden)]
pub mod __doc_utils;
