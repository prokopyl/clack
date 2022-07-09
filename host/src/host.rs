mod error;
mod info;

pub use error::HostError;
pub use info::HostInfo;

use crate::extensions::HostExtensions;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};

pub trait HostAudioProcessor<'a>: Send + 'a {}

pub trait HostMainThread<'a>: 'a {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginMainThreadHandle) {}
}

pub trait HostShared<'a>: Send + Sync {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginSharedHandle<'a>) {}

    fn request_restart(&self);
    fn request_process(&self);
    fn request_callback(&self);
}

pub trait Host<'a>: 'static {
    type AudioProcessor: HostAudioProcessor<'a> + 'a;
    type Shared: HostShared<'a> + 'a;
    type MainThread: HostMainThread<'a> + 'a;

    #[inline]
    #[allow(unused)]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared) {}
}
