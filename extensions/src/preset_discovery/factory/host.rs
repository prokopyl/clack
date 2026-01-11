use super::*;
use crate::preset_discovery::prelude::*;
use std::iter::FusedIterator;

impl<'a> PresetDiscoveryFactory<'a> {
    /// Returns the number of providers exposed by this factory.
    pub fn provider_count(&self) -> u32 {
        let Some(count) = self.0.get().count else {
            return 0;
        };

        // SAFETY: This type enforces the contained pointer is still valid.
        unsafe { count(self.0.as_ptr()) }
    }

    /// Returns the [`ProviderDescriptor`] of the provider that is assigned the given index.
    ///
    /// Hosts will usually call this method repeatedly with every index from 0 to the total returned
    /// by [`provider_count`](Self::provider_count), in order to discover all the providers
    /// exposed by this factory.
    ///
    /// If the given index is out of bounds, or in general does not match any given providers, this
    /// returns [`None`].
    pub fn get_provider_descriptor(&self, index: u32) -> Option<&'a ProviderDescriptor> {
        let get_descriptor = self.0.get().get_descriptor?;

        // SAFETY: This type enforces the contained pointer is still valid.
        let descriptor = unsafe { get_descriptor(self.0.as_ptr(), index) };

        // SAFETY: The CLAP spec guarantees that if non-NULL, the descriptor pointer is properly aligned and valid.
        // The descriptor is read-only and never mutated, so it is safe to convert to a shared reference.
        let descriptor = unsafe { descriptor.as_ref()? };

        // SAFETY: The CLAP spec guarantees that the contents are either NULL or point to valid C strings.
        // The lifetime of that descriptor is also tied to the factory, which this type tracks as the
        // 'a lifetime.
        let descriptor = unsafe { ProviderDescriptor::from_raw(descriptor) };

        Some(descriptor)
    }

    /// Returns an iterator over all the [`ProviderDescriptor`]s exposed by this factory.
    #[inline]
    pub fn provider_descriptors(&self) -> ProviderDescriptorsIter<'a> {
        ProviderDescriptorsIter {
            factory: *self,
            range: 0..self.provider_count(),
        }
    }
}

impl<'a> IntoIterator for PresetDiscoveryFactory<'a> {
    type Item = &'a ProviderDescriptor;
    type IntoIter = ProviderDescriptorsIter<'a>;

    #[inline]
    fn into_iter(self) -> ProviderDescriptorsIter<'a> {
        self.provider_descriptors()
    }
}

impl<'a> IntoIterator for &PresetDiscoveryFactory<'a> {
    type Item = &'a ProviderDescriptor;
    type IntoIter = ProviderDescriptorsIter<'a>;

    #[inline]
    fn into_iter(self) -> ProviderDescriptorsIter<'a> {
        self.provider_descriptors()
    }
}

/// An [`Iterator`] over all the [`ProviderDescriptor`s](ProviderDescriptor) exposed by a
/// plugin factory.
///
/// See the [`PresetDiscoveryFactory::provider_descriptors`] method that produces this iterator.
pub struct ProviderDescriptorsIter<'a> {
    factory: PresetDiscoveryFactory<'a>,
    range: core::ops::Range<u32>,
}

impl<'a> Iterator for ProviderDescriptorsIter<'a> {
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

impl ExactSizeIterator for ProviderDescriptorsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl FusedIterator for ProviderDescriptorsIter<'_> {}

impl DoubleEndedIterator for ProviderDescriptorsIter<'_> {
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
