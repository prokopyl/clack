use crate::extensions::wrapper::panic::catch_unwind;
use crate::factory::Factory;
use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr};
use std::panic::AssertUnwindSafe;
use std::ptr::NonNull;

pub use clack_common::entry::*;

mod single;

pub use single::SinglePluginEntry;

pub trait Entry: Sized + Send + Sync {
    fn new(plugin_path: &CStr) -> Option<Self>;

    fn declare_factories<'a>(&'a self, builder: &mut EntryFactoriesBuilder<'a>);
}

#[macro_export]
macro_rules! clack_export_entry {
    ($entry_type:ty) => {
        #[allow(non_upper_case_globals)]
        #[allow(unsafe_code)]
        #[no_mangle]
        pub static clap_entry: $crate::entry::PluginEntryDescriptor = {
            static HOLDER: $crate::entry::EntryHolder<$entry_type> =
                $crate::entry::EntryHolder::new();

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

            $crate::entry::PluginEntryDescriptor {
                clap_version: $crate::utils::ClapVersion::CURRENT.to_raw(),
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            }
        };
    };
}

pub struct EntryFactoriesBuilder<'a> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
}

impl<'a> EntryFactoriesBuilder<'a> {
    #[inline]
    pub(crate) fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
        }
    }

    #[inline]
    pub(crate) fn found(&self) -> *const c_void {
        self.found
            .map(|p| p.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    /// Adds a given factory implementation to the list of extensions this plugin entry supports.
    pub fn register_factory<F: Factory>(&mut self, factory: &'a F) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        if F::IDENTIFIER == self.requested {
            self.found = Some(factory.get_raw_factory_ptr())
        }

        self
    }
}

#[doc(hidden)]
pub struct EntryHolder<E> {
    inner: UnsafeCell<Option<E>>,
}

// SAFETY: TODO
unsafe impl<E> Send for EntryHolder<E> {}
unsafe impl<E> Sync for EntryHolder<E> {}

#[doc(hidden)]
impl<E: Entry> EntryHolder<E> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self, plugin_path: *const core::ffi::c_char) -> bool {
        if (*self.inner.get()).is_some() {
            return true;
        }

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
            let mut builder = EntryFactoriesBuilder::new(identifier);
            entry.declare_factories(&mut builder);
            builder.found()
        }))
        .unwrap_or(core::ptr::null())
    }
}
