use crate::extension::PluginExtensions;
use crate::host::HostHandle;
use crate::process::audio::Audio;
use crate::process::events::ProcessEvents;
use crate::process::Process;
use clack_common::process::ProcessStatus;

mod descriptor;
mod error;
mod instance;
pub(crate) mod logging;
pub mod wrapper;

pub use descriptor::*;
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

    const DESCRIPTOR: &'static PluginDescriptor;

    fn new(
        host: HostHandle<'a>,
        main_thread: &mut Self::MainThread,
        shared: &'a Self::Shared,
        sample_config: SampleConfig,
    ) -> Result<Self>;

    fn process(
        &mut self,
        process: &Process,
        audio: Audio,
        events: ProcessEvents,
    ) -> Result<ProcessStatus>;

    #[inline]
    fn deactivate(self, _main_thread: &mut Self::MainThread) {}

    #[inline]
    fn start_processing(&mut self) -> Result {
        Ok(())
    }
    #[inline]
    fn stop_processing(&mut self) {}

    #[inline]
    fn declare_extensions(_builder: &mut PluginExtensions<Self>, _shared: &Self::Shared) {}
}
