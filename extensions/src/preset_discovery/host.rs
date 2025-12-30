mod extension;
pub mod indexer;
pub(crate) mod provider;

pub use provider::{Provider, ProviderInstanceError};
use std::iter::FusedIterator;

mod metadata_receiver;
pub use extension::*;
pub use metadata_receiver::MetadataReceiver;

use super::*;

impl<'a> PresetDiscoveryFactory<'a> {
    pub fn provider_count(&self) -> u32 {
        let Some(count) = self.0.get().count else {
            return 0;
        };

        // SAFETY: TODO
        unsafe { count(self.0.as_ptr()) }
    }

    pub fn get_provider_descriptor(&self, index: u32) -> Option<&'a ProviderDescriptor> {
        let get_descriptor = self.0.get().get_descriptor?;

        // SAFETY: TODO
        let descriptor = unsafe { get_descriptor(self.0.as_ptr(), index) };

        // SAFETY: TODO
        let descriptor = unsafe { descriptor.as_ref()? };

        // SAFETY: TODO
        let descriptor = unsafe { ProviderDescriptor::from_raw(descriptor) };

        Some(descriptor)
    }

    #[inline]
    pub fn provider_descriptors(&self) -> PluginDescriptorsIter<'a> {
        PluginDescriptorsIter {
            factory: *self,
            range: 0..self.provider_count(),
        }
    }
}

/// An [`Iterator`] over all the [`PluginDescriptor`s](PluginDescriptor) exposed by a
/// plugin factory.
///
/// See the [`PluginFactory::plugin_descriptors`] method that produces this iterator.
pub struct PluginDescriptorsIter<'a> {
    factory: PresetDiscoveryFactory<'a>,
    range: core::ops::Range<u32>,
}

impl<'a> Iterator for PluginDescriptorsIter<'a> {
    type Item = &'a ProviderDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.range.next()?;

            if let Some(descriptor) = self.factory.get_provider_descriptor(next) {
                return Some(descriptor);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl ExactSizeIterator for PluginDescriptorsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl FusedIterator for PluginDescriptorsIter<'_> {}

impl DoubleEndedIterator for PluginDescriptorsIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.range.next_back()?;

            if let Some(descriptor) = self.factory.get_provider_descriptor(next) {
                return Some(descriptor);
            }
        }
    }
}
