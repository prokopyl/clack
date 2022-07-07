mod info;

use crate::extensions::HostExtensions;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
pub use info::HostInfo;
use std::sync::Arc;

// TODO: bikeshed
pub(crate) struct HostShared {
    info: HostInfo,
}

impl HostShared {
    #[inline]
    pub fn info(&self) -> &HostInfo {
        &self.info
    }
}

// TODO: rename
#[derive(Clone)]
pub struct PluginHost {
    inner: Arc<HostShared>,
}

impl PluginHost {
    #[inline]
    pub fn new(info: HostInfo) -> Self {
        Self {
            inner: Arc::new(HostShared { info }),
        }
    }

    #[inline]
    pub(crate) fn shared(&self) -> &Arc<HostShared> {
        &self.inner
    }
}

// TODO: bikeshed
pub trait AudioProcessorHoster: Send {}

pub trait MainThreadHoster<'a>: Send + 'a {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginMainThreadHandle) {}
}

pub trait SharedHoster<'a>: Send + Sync {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginSharedHandle<'a>) {}

    fn request_restart(&self);
    fn request_process(&self);
    fn request_callback(&self);
}

// TODO: rename
pub trait PluginHoster<'a>: Sized + 'static {
    type AudioProcessor: AudioProcessorHoster + 'a;
    type Shared: SharedHoster<'a> + 'a;
    type MainThread: MainThreadHoster<'a> + 'a;

    #[inline]
    #[allow(unused)]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared) {}
}
