mod info;

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
