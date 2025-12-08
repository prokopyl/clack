#![warn(missing_docs)]

//! Allows a plugin to provide a structured way of mapping parameters to shortcut pages or a hardware
//! controller.
//!
//! This is done by providing a set of remote control pages, organized by section.
//! A page contains up to 8 controls, which references parameters using their Parameter ID.
//!
//! See the [`RemoteControlsPage`] type's documentation for more information.

use crate::utils::data_from_array_buf;
use clack_common::extensions::*;
use clack_common::utils::ClapId;
use clap_sys::ext::remote_controls::*;
use std::ffi::CStr;

/// Plugin-side of the Remote Controls extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginRemoteControls(RawExtension<PluginExtensionSide, clap_plugin_remote_controls>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginRemoteControls {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_REMOTE_CONTROLS, CLAP_EXT_REMOTE_CONTROLS_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Host-side of the Remote Controls extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostRemoteControls(RawExtension<HostExtensionSide, clap_host_remote_controls>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostRemoteControls {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_REMOTE_CONTROLS, CLAP_EXT_REMOTE_CONTROLS_COMPAT];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// A remote control page.
///
/// This type is tied to the lifetime `'a` of the host buffer the name strings are actually written into.
#[derive(Copy, Clone)]
pub struct RemoteControlsPage<'a> {
    /// The name of the section the page resides in.
    /// This may be used by hosts to organize and display pages in a hierarchical manner.
    pub section_name: &'a [u8],
    /// The unique identifier of the page.
    pub page_id: ClapId,
    /// The display name of the page.
    pub page_name: &'a [u8],
    /// The IDs of parameters each slot is mapped to.
    /// Use `None` to leave a specific slot empty.
    pub param_ids: [Option<ClapId>; 8],
    /// Specifies whether the page is part of the plugin/device, or specific to a preset.
    /// If `true`, then this page is specific to the current preset.
    pub is_for_preset: bool,
}

impl<'a> RemoteControlsPage<'a> {
    /// Reads remote control page information from a raw, C-FFI compatible struct.
    ///
    /// Returns `None` if the `page_id` field is an invalid ID.
    #[inline]
    pub fn from_raw(raw: &'a clap_remote_controls_page) -> Option<Self> {
        Some(Self {
            section_name: data_from_array_buf(&raw.section_name),
            page_id: ClapId::from_raw(raw.page_id)?,
            page_name: data_from_array_buf(&raw.page_name),
            param_ids: raw.param_ids.map(ClapId::from_raw),
            is_for_preset: raw.is_for_preset,
        })
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use crate::remote_controls::PluginRemoteControls;
    use clack_host::extensions::prelude::*;
    use clap_sys::ext::remote_controls::clap_remote_controls_page;
    use clap_sys::id::clap_id;
    use std::mem::MaybeUninit;

    /// A buffer for plugins to write remote control page information into.
    ///
    /// This is to be passed to the [`PluginRemoteControls::get`] method, which allows the host to write into
    /// it and to then retrieve a valid [`RemoteControlsPage`] from it.
    #[derive(Clone)]
    pub struct RemoteControlsPageBuffer {
        inner: MaybeUninit<clap_remote_controls_page>,
    }

    impl RemoteControlsPageBuffer {
        /// Creates a new, empty track info buffer.
        #[inline]
        pub const fn new() -> Self {
            Self {
                inner: MaybeUninit::zeroed(),
            }
        }
    }

    impl Default for RemoteControlsPageBuffer {
        #[inline]
        fn default() -> Self {
            Self::new()
        }
    }

    impl PluginRemoteControls {
        /// Returns the number of Remote Control pages the plugin provides.
        #[inline]
        pub fn count(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
            let Some(count) = plugin.use_extension(&self.0).count else {
                return 0;
            };

            // SAFETY: This type guarantees the function pointer is valid, and
            // PluginMainThreadHandle guarantees the plugin pointer is valid
            unsafe { count(plugin.as_raw()) }
        }

        /// Request the plugin to write the information of the Remote Control page at the given `index`,
        /// into the given [`RemoteControlsPageBuffer`].
        ///
        /// If successful, a valid [`RemoteControlsPage`] (referencing the given buffer) is returned.
        /// Otherwise, [`None`] is returned.
        #[inline]
        pub fn get<'a>(
            &self,
            plugin: &mut PluginMainThreadHandle,
            index: u32,
            buffer: &'a mut RemoteControlsPageBuffer,
        ) -> Option<RemoteControlsPage<'a>> {
            let get = plugin.use_extension(&self.0).get?;

            // SAFETY: This type guarantees the function pointer is valid, and
            // PluginMainThreadHandle guarantees the plugin pointer is valid
            let success = unsafe { get(plugin.as_raw(), index, buffer.inner.as_mut_ptr()) };
            if !success {
                return None;
            }

            // SAFETY: per the clap spec, if 'get' returns true (which we checked above), then the buffer is initialized.
            // Worst case the buffer is zeroed, which is always valid for this type.
            let raw_page = unsafe { buffer.inner.assume_init_ref() };

            RemoteControlsPage::from_raw(raw_page)
        }
    }

    /// Implementation of the Host-side of the Remote Controls extension.
    pub trait HostRemoteControlsImpl {
        /// Informs the host that the Remote Control pages provided by the plugin have changed and need to be re-scanned.
        fn changed(&mut self);
        /// Suggests the host to display/activate a given page, e.g. because it corresponds to what the user
        /// is currently editing in the plugin's GUI.
        fn suggest_page(&mut self, page_id: ClapId);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostRemoteControls
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostRemoteControlsImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_remote_controls {
                changed: Some(changed::<H>),
                suggest_page: Some(suggest_page::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn changed<H>(host: *const clap_host)
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostRemoteControlsImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn suggest_page<H>(host: *const clap_host, page_id: clap_id)
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostRemoteControlsImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            let id = ClapId::from_raw(page_id)
                .ok_or(HostWrapperError::InvalidParameter("Invalid page ID"))?;

            host.main_thread().as_mut().suggest_page(id);
            Ok(())
        });
    }
}
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use crate::utils::write_to_array_buf;
    use clack_plugin::extensions::prelude::*;
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;

    impl HostRemoteControls {
        /// Informs the host that the Remote Control pages provided by the plugin have changed and need to be re-scanned.
        #[inline]
        pub fn changed(&self, plugin: &mut HostMainThreadHandle) {
            if let Some(changed) = plugin.use_extension(&self.0).changed {
                // SAFETY: This type guarantees the function pointer is valid, and
                // HostMainThreadHandle guarantees the host pointer is valid
                unsafe { changed(plugin.as_raw()) }
            }
        }

        /// Suggests the host to display/activate a given page, e.g. because it corresponds to what the user
        /// is currently editing in the plugin's GUI.
        pub fn suggest_page(&self, plugin: &mut HostMainThreadHandle, page_id: ClapId) {
            if let Some(suggest_page) = plugin.use_extension(&self.0).suggest_page {
                // SAFETY: This type guarantees the function pointer is valid, and
                // HostMainThreadHandle guarantees the host pointer is valid
                unsafe { suggest_page(plugin.as_raw(), page_id.get()) }
            }
        }
    }

    /// A helper type that allows to safely write a [`RemoteControlsPage`] to an uninitialized host-provided
    /// buffer.
    ///
    /// This type wraps a pointer to a host-provided, potentially uninitialized track info buffer,
    /// and exposes the [`set`](RemoteControlsPageWriter::set) method to safely write into it.
    ///
    /// This type also tracks whether anything was written into the buffer at all, so it can report
    /// success back to the host.
    pub struct RemoteControlsPageWriter<'a> {
        buffer: *mut clap_remote_controls_page,
        _buffer: PhantomData<&'a mut clap_remote_controls_page>,
        is_set: bool,
    }

    impl<'a> RemoteControlsPageWriter<'a> {
        /// Wraps a given mutable reference to a potentially initialized C-FFI compatible buffer.
        #[inline]
        pub const fn from_raw_buf(buffer: &'a mut MaybeUninit<clap_remote_controls_page>) -> Self {
            // SAFETY: Coming from a &mut guarantees the pointer is valid for writes, non-null and aligned.
            unsafe { Self::from_raw(buffer.as_mut_ptr()) }
        }

        /// Wraps a given pointer to a C-FFI compatible buffer.
        ///
        /// # Safety
        ///
        /// Callers must ensure the pointer must be valid for writes for the lifetime of `'a`. It
        /// must also be non-null and well-aligned.
        #[inline]
        pub const unsafe fn from_raw(ptr: *mut clap_remote_controls_page) -> Self {
            Self {
                buffer: ptr,
                _buffer: PhantomData,
                is_set: false,
            }
        }

        /// Writes the given `remote_controls_page` into the buffer this type wraps.
        pub fn set(&mut self, remote_controls_page: &RemoteControlsPage<'_>) {
            use core::ptr::write;

            let buf = self.buffer;

            // SAFETY: This type ensures the buf pointer is valid for writes and well-aligned.
            unsafe {
                write_to_array_buf(
                    &raw mut (*buf).section_name,
                    remote_controls_page.section_name,
                );
                write_to_array_buf(&raw mut (*buf).page_name, remote_controls_page.page_name);
                write(&raw mut (*buf).page_id, remote_controls_page.page_id.get());
                write(
                    &raw mut (*buf).is_for_preset,
                    remote_controls_page.is_for_preset,
                );
                write(
                    &raw mut (*buf).param_ids,
                    remote_controls_page.param_ids.map(ClapId::optional_to_raw),
                );
            }

            self.is_set = true;
        }
    }

    /// Implementation of the Plugin-side of the Remote Controls extension.
    pub trait PluginRemoteControlsImpl {
        /// Returns the number of Remote Control pages the plugin provides.
        fn count(&mut self) -> u32;
        /// Writes the information of the Remote Control page at the given `index` into the given `writer`.
        ///
        /// If unsuccessful (e.g. if `index` is out of bounds), the `writer` can be ignored, which
        /// will report the lack of a Remote Control page back to the host.
        fn get(&mut self, index: u32, writer: &mut RemoteControlsPageWriter);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P> ExtensionImplementation<P> for PluginRemoteControls
    where
        P: for<'a> Plugin<MainThread<'a>: PluginRemoteControlsImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_remote_controls {
                get: Some(get::<P>),
                count: Some(count::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn count<P>(plugin: *const clap_plugin) -> u32
    where
        P: for<'a> Plugin<MainThread<'a>: PluginRemoteControlsImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| Ok(plugin.main_thread().as_mut().count()))
            .unwrap_or(0)
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get<P>(
        plugin: *const clap_plugin,
        index: u32,
        buf: *mut clap_remote_controls_page,
    ) -> bool
    where
        P: for<'a> Plugin<MainThread<'a>: PluginRemoteControlsImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            let mut writer = RemoteControlsPageWriter::from_raw(buf);
            plugin.main_thread().as_mut().get(index, &mut writer);
            Ok(writer.is_set)
        })
        .unwrap_or(false)
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
