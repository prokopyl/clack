use crate::factory::plugin::{PluginFactory, PluginFactoryImpl};
use crate::factory::PluginFactories;
use crate::host::HostInfo;
use crate::plugin::descriptor::{PluginDescriptorWrapper, RawPluginDescriptor};
use crate::plugin::{Plugin, PluginInstance};
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;

use crate::extensions::wrapper::panic::catch_unwind;
pub use clack_common::bundle::*;

pub trait PluginEntry: Sized {
    fn new(plugin_path: &CStr) -> Option<Self>;

    fn declare_factories(&self, builder: &mut PluginFactories);
}

#[macro_export]
macro_rules! clack_export_entry {
    ($plugin_ty:ty) => {
        #[allow(non_upper_case_globals)]
        #[allow(unsafe_code)]
        #[no_mangle]
        pub static clap_entry: $crate::bundle::PluginEntryDescriptor = {
            static HOLDER: $crate::bundle::EntryHolder<$plugin_ty> =
                $crate::bundle::EntryHolder::new();

            unsafe extern "C" fn init(plugin_path: *const ::core::ffi::c_char) -> bool {
                HOLDER.init(plugin_path)
            }

            unsafe extern "C" fn deinit() {
                HOLDER.de_init()
            }

            unsafe extern "C" fn get_factory(
                identifier: *const ::core::ffi::c_char,
            ) -> *const ::core::ffi::c_void {
                HOLDER.get_factory(identifier)
            }

            $crate::bundle::PluginEntryDescriptor {
                clap_version: $crate::utils::ClapVersion::CURRENT.to_raw(),
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            }
        };
    };
}

#[doc(hidden)]
pub struct EntryHolder<E> {
    inner: UnsafeCell<Option<E>>,
}

// SAFETY: TODO
unsafe impl<E> Send for EntryHolder<E> {}
unsafe impl<E> Sync for EntryHolder<E> {}

#[doc(hidden)]
impl<E: PluginEntry> EntryHolder<E> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self, plugin_path: *const core::ffi::c_char) -> bool {
        let plugin_path = CStr::from_ptr(plugin_path);
        let entry = catch_unwind(|| E::new(plugin_path));

        if let Ok(Some(entry)) = entry {
            *self.inner.get() = Some(entry);
            true
        } else {
            false
        }
    }

    pub unsafe fn de_init(&self) {
        let _ = catch_unwind(AssertUnwindSafe(|| *self.inner.get() = None));
    }

    pub unsafe fn get_factory(
        &self,
        identifier: *const core::ffi::c_char,
    ) -> *const core::ffi::c_void {
        if identifier.is_null() {
            return core::ptr::null();
        }

        let Some(entry) = &*self.inner.get() else { return core::ptr::null() };
        let identifier = CStr::from_ptr(identifier);

        catch_unwind(AssertUnwindSafe(|| {
            let mut builder = PluginFactories::new(identifier);
            entry.declare_factories(&mut builder);
            builder.found()
        }))
        .unwrap_or(core::ptr::null())
    }
}

pub struct SinglePluginEntry<'a, P: Plugin<'a>> {
    plugin_factory: PluginFactory<SinglePluginFactory<'a, P>>,
}

impl<'a, P: Plugin<'a>> PluginEntry for SinglePluginEntry<'a, P> {
    fn new(_plugin_path: &CStr) -> Option<Self> {
        Some(Self {
            plugin_factory: PluginFactory::new(SinglePluginFactory {
                descriptor: PluginDescriptorWrapper::new(P::get_descriptor()),
                _plugin: PhantomData,
            }),
        })
    }

    #[inline]
    fn declare_factories(&self, builder: &mut PluginFactories) {
        builder.register(&self.plugin_factory);
    }
}

struct SinglePluginFactory<'a, P: Plugin<'a>> {
    descriptor: PluginDescriptorWrapper,
    _plugin: PhantomData<AssertUnwindSafe<&'a P>>,
}

impl<'a, P: Plugin<'a>> PluginFactoryImpl<'a> for SinglePluginFactory<'a, P> {
    #[inline]
    fn plugin_count(&self) -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(&self, index: u32) -> Option<&RawPluginDescriptor> {
        match index {
            0 => Some(self.descriptor.get_raw()),
            _ => None,
        }
    }

    #[inline]
    fn create_plugin(
        &'a self,
        host_info: HostInfo<'a>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'a>> {
        if plugin_id == self.descriptor.descriptor().id() {
            Some(PluginInstance::new::<P>(
                host_info,
                self.descriptor.get_raw(),
            ))
        } else {
            None
        }
    }
}
