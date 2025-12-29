use crate::preset_discovery::plugin::indexer::Indexer;

pub struct ProviderWrapper<P> {
    pub(crate) inner: P,
}
