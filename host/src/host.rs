mod info;

use crate::extensions::HostExtensions;
use crate::plugin::{PluginMainThread, PluginShared};
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

#[allow(unused)] // For default impls
pub trait SharedHoster: Send + Sync {
    #[inline]
    fn instantiated(&mut self, instance: PluginShared) {}

    fn request_restart(&self);
    fn request_process(&self);
    fn request_callback(&self);
}

// TODO
#[allow(unused)] // For default impls
pub trait PluginHoster<'a>: Sized + 'a {
    type AudioProcessor: AudioProcessorHoster + 'a;
    type Shared: SharedHoster + 'a;

    #[inline]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared) {}

    #[inline]
    fn instantiated(&mut self, instance: PluginMainThread) {}
}
