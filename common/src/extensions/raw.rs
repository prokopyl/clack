#![deny(missing_docs)]
use crate::extensions::{ExtensionSide, HostExtensionSide, PluginExtensionSide};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A raw extension pointer.
///
/// This type is used by extension implementations to abstract away most of the safety concerns of
/// using extension pointers.
///
/// Using the `S` type parameter with either [`PluginExtensionSide`] or [`HostExtensionSide`], this
/// can be used as either a plugin-side extension or a host-side extension, respectively.
///
/// The `T` type parameter is the extension struct this points to (e.g. `clap_plugin_state`), or
/// `()` if the extension pointer is untyped.
///
/// Internally, this holds two pointers: a pointer to the actual extension struct, and a pointer
/// to the matching `clap_plugin` or `clap_host`. These are compared on each use to make sure
/// that the right extensions are always called with the right instance.
///
/// This type does not directly track the lifetimes of its pointers at all, they may become dangling
/// at any time. However, it is part of the safety contract of this type that the extension pointer
/// *MUST* be valid as long as the matching `clap_plugin` or `clap_host` still is, as per the CLAP
/// specification.
///
/// This property is then extrapolated so that if any valid `clap_plugin` or `clap_host` reference,
/// that also matches the pointer stored in this type, is encountered, then it must mean that the
/// inner extension pointer is also valid, and can be safely de-referenced.
///
/// To safely get valid references to the extension struct out of this type, you have to use the
/// `use_extension()` method on one of the handle types:
///
/// * For plugin-side extensions (from the `clack-host` crate): `PluginSharedHandle`,
///   `PluginMainThreadHandle`, or `PluginAudioThreadHandle`;
/// * For host-side extensions (from the `clack-plugin` crate): `HostSharedHandle`,
///   `HostMainThreadHandle`, or `HostAudioThreadHandle`.
///
/// This pointer type is only useful for *consuming* extension pointers. For producing extension
/// pointers from an extension implementation, use the [`RawExtensionImplementation`] type instead.  
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RawExtension<S: ExtensionSide, T = ()> {
    extension_ptr: NonNull<T>,
    host_or_plugin_ptr: NonNull<c_void>, // Can be either clap_host or clap_plugin
    _side: PhantomData<fn() -> S>,
}

// SAFETY: this is just a couple of pointers, and the type doesn't care being used on any thread.
// Thread-safety is enforced by the plugin handle type that is passed to the methods.
unsafe impl<S: ExtensionSide, T> Send for RawExtension<S, T> {}
// SAFETY: same as above.
unsafe impl<S: ExtensionSide, T> Sync for RawExtension<S, T> {}

impl<S: ExtensionSide, T> RawExtension<S, T> {
    /// Returns the raw pointer to the extension struct.
    ///
    /// This pointer may be dangling at any time.
    ///
    /// See the [`RawExtension`] type documentation for more information on how to get a valid
    /// reference to the extension struct instead.
    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        self.extension_ptr
    }
}

impl<S: ExtensionSide> RawExtension<S, ()> {
    /// Casts this raw extension pointer into an extension pointer of a given type.
    ///
    /// # Safety
    ///
    /// Users *must* ensure that `T` matches the actual type behind the pointer.
    pub unsafe fn cast<T>(&self) -> RawExtension<S, T> {
        RawExtension {
            extension_ptr: self.extension_ptr.cast(),
            host_or_plugin_ptr: self.host_or_plugin_ptr,
            _side: PhantomData,
        }
    }
}

impl<T> RawExtension<PluginExtensionSide, T> {
    /// Creates a raw plugin-side extension pointer from a pointer to the extension data, and a
    /// pointer to the plugin instance.
    ///
    /// # Safety
    ///
    /// The user *must* ensure the `extension_ptr` is and remains valid for the lifetime of the
    /// plugin instance.
    ///
    /// The given `plugin_ptr` however doesn't have to be valid, and may be dangling.
    pub unsafe fn from_raw_plugin_extension(
        extension_ptr: NonNull<T>,
        plugin_ptr: NonNull<clap_plugin>,
    ) -> Self {
        Self {
            extension_ptr,
            host_or_plugin_ptr: plugin_ptr.cast(),
            _side: PhantomData,
        }
    }

    /// Returns the raw pointer to the plugin instance the extension originated from.
    ///
    /// This pointer may be dangling at any time, and is not meant to be de-referenced, only
    /// compared to.
    /// See the [`RawExtension`] type documentation for more information.
    pub fn plugin_ptr(&self) -> NonNull<clap_plugin> {
        self.host_or_plugin_ptr.cast()
    }
}

impl<T> RawExtension<HostExtensionSide, T> {
    /// Creates a raw host-side extension pointer from a pointer to the extension data, and a
    /// pointer to the plugin instance.
    ///
    /// # Safety
    ///
    /// The user *must* ensure the `extension_ptr` is and remains valid for the lifetime of the
    /// plugin instance.
    ///
    /// The given `host_ptr` however doesn't have to be valid, and may be dangling.
    pub unsafe fn from_raw_host_extension(
        extension_ptr: NonNull<T>,
        host_ptr: NonNull<clap_host>,
    ) -> Self {
        Self {
            extension_ptr,
            host_or_plugin_ptr: host_ptr.cast(),
            _side: PhantomData,
        }
    }

    /// Returns the raw pointer to the host instance the extension originated from.
    ///
    /// This pointer may be dangling at any time, and is not meant to be de-referenced, only
    /// compared to.
    /// See the [`RawExtension`] type documentation for more information.
    pub fn host_ptr(&self) -> NonNull<clap_host> {
        self.host_or_plugin_ptr.cast()
    }
}

/// A pointer to an extension's implementation struct.
///
/// This type is used to type-erase a reference to the implementation struct of an extension before
/// transferring it to the consumer through `get_extension`.
///
/// This type is also used to enforce the extension pointer is always valid, as extension
/// implementations produced from Clack always come from statics and therefore always have the
/// `'static` lifetime.
///
/// Note this is *not* the case for all CLAP plugins: extension pointers are only guaranteed to be
/// valid for the lifetime of the instance (plugin or host) they originate from.
///
/// Use the [`RawExtension`] type to properly use and track the lifetime of extension pointers on
/// the consumer-side.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RawExtensionImplementation {
    inner: NonNull<c_void>,
}

impl RawExtensionImplementation {
    /// Creates a new pointer from a `'static` reference to an implementation struct.
    pub const fn new<I>(implementation: &'static I) -> Self {
        Self {
            // SAFETY: pointer comes from a reference, so it's guaranteed to always be valid.
            inner: unsafe { NonNull::new_unchecked(implementation as *const _ as *mut _) },
        }
    }

    /// Returns the raw pointer to the extension implementation struct.
    ///
    /// This pointer is always valid and never dangling, as it originates from a `'static`
    /// reference.
    pub const fn as_ptr(&self) -> NonNull<c_void> {
        self.inner
    }
}

impl Debug for RawExtensionImplementation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawExtensionImplementation({:p})", self.inner)
    }
}
