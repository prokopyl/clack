use crate::factory::{Factory, RawFactoryPointer};
use crate::plugin::PluginDescriptor;
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use std::ffi::CStr;
use std::iter::FusedIterator;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginFactory<'a>(RawFactoryPointer<'a, clap_plugin_factory>);

// SAFETY: TODO
unsafe impl<'a> Factory<'a> for PluginFactory<'a> {
    const IDENTIFIERS: &'static [&'static CStr] = &[CLAP_PLUGIN_FACTORY_ID];
    type Raw = clap_plugin_factory;

    #[inline]
    unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self {
        Self(raw)
    }
}

impl<'a> PluginFactory<'a> {
    /// Returns this factory as a raw pointer to its C-FFI compatible raw CLAP structure
    #[inline]
    pub const fn raw(&self) -> RawFactoryPointer<'a, clap_plugin_factory> {
        self.0
    }

    /// Returns the number of plugin descriptors exposed by this plugin factory.
    #[inline]
    pub fn plugin_count(&self) -> u32 {
        match self.0.get().get_plugin_count {
            None => 0,
            // SAFETY: this type ensures the function pointer is valid
            Some(count) => unsafe { count(self.0.as_ptr()) },
        }
    }

    /// Returns the [`PluginDescriptor`s](PluginDescriptor) exposed by this plugin
    /// factory at a given index, or `None` if there is no plugin descriptor at the given index.
    ///
    /// Implementations on the plugin-side *should* return a descriptor for any index strictly less
    /// than [`plugin_count`](PluginFactory::plugin_count), but this is not a guarantee.
    ///
    /// See also the [`plugin_descriptors`](PluginFactory::plugin_descriptors) method for a
    /// convenient iterator of all the plugin descriptors exposed by this factory.
    #[inline]
    pub fn plugin_descriptor(&self, index: u32) -> Option<&'a PluginDescriptor> {
        let get_plugin_descriptor = self.0.get().get_plugin_descriptor?;
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { get_plugin_descriptor(self.0.as_ptr(), index).as_ref() }
            // SAFETY: this descriptor is guaranteed to be valid by the spec
            .map(|d| unsafe { PluginDescriptor::from_raw(d) })
    }

    /// Returns an iterator of all the [`PluginDescriptor`s](PluginDescriptor) exposed by this
    /// plugin factory.
    ///
    /// For convenience, the [`&PluginFactory`](PluginFactory) type implements the
    /// [`IntoIterator`] trait, which also returns this iterator.
    ///
    /// See also the [`plugin_descriptor`](PluginFactory::plugin_descriptor) method to retrieve
    /// a plugin descriptor at a specific index.
    #[inline]
    pub fn plugin_descriptors(&self) -> PluginDescriptorsIter<'a> {
        PluginDescriptorsIter {
            factory: *self,
            range: 0..self.plugin_count(),
        }
    }
}

/// Returns an iterator of all the [`PluginDescriptor`s](PluginDescriptor) exposed by this plugin
/// factory.
impl<'a> IntoIterator for PluginFactory<'a> {
    type Item = &'a PluginDescriptor;
    type IntoIter = PluginDescriptorsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.plugin_descriptors()
    }
}

/// An [`Iterator`] over all the [`PluginDescriptor`s](PluginDescriptor) exposed by a
/// plugin factory.
///
/// See the [`PluginFactory::plugin_descriptors`] method that produces this iterator.
pub struct PluginDescriptorsIter<'a> {
    factory: PluginFactory<'a>,
    range: core::ops::Range<u32>,
}

impl<'a> Iterator for PluginDescriptorsIter<'a> {
    type Item = &'a PluginDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.range.next()?;

            if let Some(descriptor) = self.factory.plugin_descriptor(next) {
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

            if let Some(descriptor) = self.factory.plugin_descriptor(next) {
                return Some(descriptor);
            }
        }
    }
}
