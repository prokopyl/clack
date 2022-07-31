#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

//! A safe interface for implementing CLAP Host functionality. See
//! [`test-host`](https://github.com/prokopyl/clack/tree/main/test-host)
//! for a basic implementation.
//!
//! In a nutshell, three aptly-named traits are provided:
//! * [`HostAudioProcessor`](host::HostAudioProcessor) represents the audio
//! processing context.
//! * [`HostMainThread`](host::HostMainThread) is the main context, where
//! the host performs things like GUI and converting float values into
//! `String`s.
//! * [`HostShared`](host::HostShared) is shared between the two other
//! threads by immutable reference. By taking advantage of interior
//! mutability (with an [`AtomicI32`](std::sync::atomic::AtomicI32) or
//! [`mpsc::channel`](std::sync::mpsc::channel) for example), it can be
//! used to pass values and messages between the audio and main contexts.

extern crate core;

pub mod bundle;
pub mod extensions;
pub mod factory;
pub mod host;
pub mod instance;
pub mod plugin;
pub mod wrapper;

pub use clack_common::events;
pub use clack_common::ports;
pub use clack_common::process;
pub use clack_common::stream;
pub use clack_common::utils;
pub use clack_common::version;
