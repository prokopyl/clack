#![deny(missing_docs)]

//! Factory types and associated utilities.
//!
//! In CLAP, factories are singleton objects exposed by the [plugin bundle](crate::bundle)'s
//! entry point, which can in turn expose various functionalities.
//!
//! Each factory type has a standard, unique [identifier](FactoryPointer::IDENTIFIER), which allows hosts
//! to query plugins for known factory type implementations.
//!
//! In Clack, pointers to factories are represented by the [`FactoryPointer`] trait.
//!
//! See the [`PluginBundle::get_factory`](crate::bundle::PluginBundle::get_factory) method to
//! retrieve a given factory type from a plugin bundle.
//!
//! The main factory type (and, at the time of this writing, the only stable standard one), is the
//! [`PluginFactory`], which enables hosts to list all the plugin implementations present in this
//! bundle using their [`PluginDescriptor`].
//!
//! See the [`PluginFactory`]'s type documentation for more detail and examples on how to
//! list plugins.

use crate::plugin::PluginInstanceError;
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::ffi::{CStr, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;

mod plugin_descriptor;
pub use plugin_descriptor::*;

/// A custom factory pointer type.
///
/// # Safety
///
/// Types implementing this trait **MUST** be the exact same C-FFI representation as the CLAP
/// factory struct matching the factory's [`IDENTIFIER`](FactoryPointer::IDENTIFIER).
pub unsafe trait FactoryPointer<'a>: Sized + 'a {
    /// The standard identifier for this factory.
    const IDENTIFIER: &'static CStr;
    /// Creates a new factory pointer of this type from a raw factory pointer.
    ///
    /// # Safety
    ///
    /// The caller *MUST* ensure the given pointer points to the same type that is expected by this
    /// pointer type. (e.g. the given pointer must point to a `clap_plugin_factory` to create a [`PluginFactory`]).
    unsafe fn from_raw(raw: NonNull<c_void>) -> Self;
}

/// A factory pointer that exposes a list of [`PluginDescriptor`s](PluginDescriptor).
///
/// # Example
///
///```
/// use clack_host::prelude::PluginBundle;
///
/// # mod diva { include!("./bundle/diva_stub.rs"); }
/// # let bundle = unsafe { PluginBundle::load_from_raw(&diva::DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so").unwrap() };
/// # /*
/// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
/// # */
/// // Fetch the PluginFactory from the bundle, if present
/// let plugin_factory = bundle.get_plugin_factory().unwrap();
///
/// println!("The bundle exposes {} plugins:", plugin_factory.plugin_count());
///
/// for plugin_descriptor in plugin_factory {
///    println!("- {}", plugin_descriptor.name().unwrap().to_string_lossy());
///    println!("\t ID: {}", plugin_descriptor.id().unwrap().to_string_lossy());
///    
///    let features: Vec<_> = plugin_descriptor.features().map(|f| f.to_string_lossy()).collect();
///    println!("\t Features: [{}]", features.join(", "));
/// }
/// ```
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PluginFactory<'a> {
    inner: *const clap_plugin_factory,
    _lifetime: PhantomData<&'a clap_plugin_factory>,
}

// SAFETY: This takes a clap_plugin_factory pointer, which matches CLAP_PLUGIN_FACTORY_ID
unsafe impl<'a> FactoryPointer<'a> for PluginFactory<'a> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;

    #[inline]
    unsafe fn from_raw(raw: NonNull<c_void>) -> Self {
        Self {
            inner: raw.as_ptr() as *const _,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> PluginFactory<'a> {
    /// Returns the number of plugin descriptors exposed by this plugin factory.
    #[inline]
    pub fn plugin_count(&self) -> u32 {
        // SAFETY: no special safety considerations
        match unsafe { (*self.inner).get_plugin_count } {
            None => 0,
            // SAFETY: this type ensures the function pointer is valid
            Some(count) => unsafe { count(self.inner) },
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
    pub fn plugin_descriptor(&self, index: u32) -> Option<PluginDescriptor<'a>> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (*self.inner).get_plugin_descriptor?(self.inner, index).as_ref() }
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
            count: self.plugin_count(),
            current_index: 0,
        }
    }

    /// # Safety
    ///
    /// User must pass a valid clap_host pointer, which has to stay valid for the lifetime of the
    /// plugin.
    pub(crate) unsafe fn create_plugin(
        &self,
        plugin_id: &CStr,
        host: *const clap_host,
    ) -> Result<NonNull<clap_plugin>, PluginInstanceError> {
        NonNull::new((*self.inner)
            .create_plugin
            .ok_or(PluginInstanceError::NullFactoryCreatePluginFunction)?(
            self.inner,
            host,
            plugin_id.as_ptr(),
        ) as *mut clap_plugin)
        .ok_or(PluginInstanceError::PluginNotFound)
    }
}

/// An [`Iterator`] over all the [`PluginDescriptor`s](PluginDescriptor) exposed by a
/// plugin factory.
///
/// See the [`PluginFactory::plugin_descriptors`] method that produces this iterator.
pub struct PluginDescriptorsIter<'a> {
    factory: PluginFactory<'a>,
    current_index: u32,
    count: u32,
}

impl<'a> Iterator for PluginDescriptorsIter<'a> {
    type Item = PluginDescriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_index >= self.count {
                return None;
            }

            let descriptor = self.factory.plugin_descriptor(self.current_index);
            self.current_index += 1;

            // Skip all none-returning indexes
            if let Some(d) = descriptor {
                return Some(d);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count as usize, Some(self.count as usize))
    }
}

/// Returns an iterator of all the [`PluginDescriptor`s](PluginDescriptor) exposed by this plugin
/// factory.
impl<'a> IntoIterator for PluginFactory<'a> {
    type Item = PluginDescriptor<'a>;
    type IntoIter = PluginDescriptorsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.plugin_descriptors()
    }
}
