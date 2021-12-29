use crate::extension::ExtensionDeclarations;
use crate::host::HostHandle;
use crate::process::audio::Audio;
use crate::process::events::ProcessEvents;
use crate::process::Process;
use clap_audio_common::process::ProcessStatus;
use clap_sys::{
    plugin::{clap_plugin_descriptor, CLAP_PLUGIN_AUDIO_EFFECT},
    version::CLAP_VERSION,
};

mod error;
mod instance;
mod logging;
pub mod wrapper;
pub use error::{PluginError, Result};
pub use instance::*;

pub trait PluginShared<'a>: Sized + Send + Sync + 'a {
    fn new(host: HostHandle<'a>) -> Result<Self>;
}

impl<'a> PluginShared<'a> for () {
    #[inline]
    fn new(_host: HostHandle<'a>) -> Result<Self> {
        Ok(())
    }
}

pub trait PluginMainThread<'a, S>: Sized + 'a {
    fn new(host: HostHandle<'a>, shared: &S) -> Result<Self>;

    #[inline]
    fn on_main_thread(&mut self) {}
}

impl<'a, S> PluginMainThread<'a, S> for () {
    #[inline]
    fn new(_host: HostHandle<'a>, _shared: &S) -> Result<Self> {
        Ok(())
    }
}

// TOOD: bikeshed
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub struct SampleConfig {
    pub sample_rate: f64,
    pub min_sample_count: u32,
    pub max_sample_count: u32,
}

pub trait Plugin<'a>: Sized + Send + Sync + 'a {
    type Shared: PluginShared<'a>;
    type MainThread: PluginMainThread<'a, Self::Shared>;

    const ID: &'static [u8]; // TODO: handle null-terminating stuff safely

    fn new(
        host: HostHandle<'a>,
        main_thread: &mut Self::MainThread,
        shared: &'a Self::Shared,
        sample_config: SampleConfig,
    ) -> Result<Self>;

    #[inline]
    fn start_processing(&mut self) -> Result {
        Ok(())
    }
    #[inline]
    fn stop_processing(&mut self) {}

    fn process(
        &mut self,
        process: &Process,
        audio: Audio,
        events: ProcessEvents,
    ) -> Result<ProcessStatus>;

    #[inline]
    fn declare_extensions(_builder: &mut ExtensionDeclarations<Self>, _shared: &Self::Shared) {}

    const DESCRIPTOR: &'static PluginDescriptor = &PluginDescriptor(clap_plugin_descriptor {
        clap_version: CLAP_VERSION,
        id: Self::ID.as_ptr() as *const i8,
        name: EMPTY.as_ptr() as *const i8,
        vendor: EMPTY.as_ptr() as *const i8,
        url: EMPTY.as_ptr() as *const i8,
        manual_url: EMPTY.as_ptr() as *const i8,
        version: EMPTY.as_ptr() as *const i8,
        description: EMPTY.as_ptr() as *const i8,
        keywords: EMPTY.as_ptr() as *const i8,
        support_url: EMPTY.as_ptr() as *const i8,
        // FIXME: Why is this u64 but plugin types are i32?
        plugin_type: CLAP_PLUGIN_AUDIO_EFFECT as u64, // TODO
    });
}

pub struct PluginDescriptor(pub(crate) clap_plugin_descriptor);

const EMPTY: &[u8] = b"\0"; // TODO
