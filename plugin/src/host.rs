//! Types and handles for plugins to interact with the host.

use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clack_common::utils::ClapVersion;
use clap_sys::host::clap_host;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

/// Various information about the host, provided at plugin instantiation time.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct HostInfo<'a> {
    raw: NonNull<clap_host>,
    _lifetime: PhantomData<&'a clap_host>,
}

impl<'a> HostInfo<'a> {
    /// Creates a new [`HostInfo`] type from a given raw, C FFI compatible pointer.
    ///
    /// # Safety
    /// Pointer must be valid for the duration of the `'a` lifetime. Moreover, the contents of
    /// the `clap_host` struct must all also be valid.
    #[inline]
    pub unsafe fn from_raw(raw: NonNull<clap_host>) -> Self {
        Self {
            raw,
            _lifetime: PhantomData,
        }
    }

    /// The [`ClapVersion`] the host uses.
    #[inline]
    pub fn clap_version(&self) -> ClapVersion {
        ClapVersion::from_raw(self.as_raw().clap_version)
    }

    /// A user-friendly name for the host.
    ///
    /// This should always be set by the host.
    pub fn name(&self) -> Option<&'a CStr> {
        NonNull::new(self.as_raw().name as *mut _)
            // SAFETY: this type ensures the pointers are valid
            .map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) })
    }

    /// The host's vendor.
    ///
    /// This field is optional.
    pub fn vendor(&self) -> Option<&'a CStr> {
        NonNull::new(self.as_raw().vendor as *mut _)
            // SAFETY: this type ensures the pointers are valid
            .map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) })
    }

    /// A URL to the host's webpage.
    ///
    /// This field is optional.
    pub fn url(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the pointers are valid
        NonNull::new(self.as_raw().url as *mut _).map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) })
    }

    /// A version string for the host.
    ///
    /// This should always be set by the host.
    pub fn version(&self) -> Option<&'a CStr> {
        NonNull::new(self.as_raw().version as *mut _)
            // SAFETY: this type ensures the pointers are valid
            .map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) })
    }

    pub fn get_extension<E: Extension<ExtensionSide = HostExtensionSide>>(&self) -> Option<E> {
        // SAFETY: this type ensures the function pointers are valid
        let ext =
            unsafe { self.as_raw().get_extension?(self.raw.as_ptr(), E::IDENTIFIER.as_ptr()) };

        let ext = NonNull::new(ext as *mut _)?;
        // SAFETY: The CLAP spec guarantees that the extension lives as long as the instance.
        let raw = unsafe { RawExtension::from_raw_host_extension(ext, self.raw) };

        // SAFETY: pointer comes from the associated E::IDENTIFIER.
        unsafe { Some(E::from_raw(raw)) }
    }

    /// # Safety
    /// Some functions exposed by [`HostSharedHandle`] cannot be called until plugin is initializing
    #[inline]
    pub(crate) unsafe fn to_handle(self) -> HostSharedHandle<'a> {
        HostSharedHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        // SAFETY: this type ensures the raw pointer is valid
        unsafe { self.raw.as_ref() }
    }
}

/// A thread-safe handle to the host.
///
/// This can be used to fetch information about the host, scan available extensions, or perform
/// requests to the host.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct HostSharedHandle<'a> {
    raw: NonNull<clap_host>,
    _lifetime: PhantomData<&'a clap_host>,
}

// SAFETY: this type only safely exposes the thread-safe operations of clap_host
unsafe impl<'a> Send for HostSharedHandle<'a> {}
// SAFETY: this type only safely exposes the thread-safe operations of clap_host
unsafe impl<'a> Sync for HostSharedHandle<'a> {}

impl<'a> HostSharedHandle<'a> {
    /// Returns host information.
    #[inline]
    pub fn info(&self) -> HostInfo<'a> {
        HostInfo {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    /// Returns a raw, C FFI-compatible reference to the host handle.
    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        // SAFETY: this type enforces the pointer is valid for 'a
        unsafe { self.raw.as_ref() }
    }

    #[inline]
    pub fn as_info(&self) -> &HostInfo<'a> {
        // SAFETY: this cast is valid since both types are just a NonNull<clap_host> and repr(transparent)
        unsafe { &*(self as *const Self as *const HostInfo<'a>) }
    }

    /// Requests the host to [deactivate](crate::plugin::PluginAudioProcessor::deactivate) and then
    /// [re-activate](crate::plugin::PluginAudioProcessor::activate) the plugin.
    /// The operation may be delayed by the host.
    #[inline]
    pub fn request_restart(&self) {
        if let Some(request_restart) = self.as_raw().request_restart {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_restart(self.raw.as_ptr()) }
        }
    }

    /// Requests the host to [activate](crate::plugin::PluginAudioProcessor::activate) the plugin,
    /// and start audio processing.
    /// This is useful if you have external IO and need to wake the plugin up from "sleep".
    #[inline]
    pub fn request_process(&self) {
        if let Some(request_process) = self.as_raw().request_process {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_process(self.raw.as_ptr()) }
        }
    }

    /// Requests the host to schedule a call to the
    /// [on_main_thread](crate::plugin::PluginMainThread::on_main_thread) method on the main thread.
    #[inline]
    pub fn request_callback(&self) {
        if let Some(request_callback) = self.as_raw().request_callback {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_callback(self.raw.as_ptr()) }
        }
    }

    /// # Safety
    ///
    /// Callers *MUST* ensure this is only called on the main thread, and that they have exclusive (&mut) access.
    #[inline]
    pub unsafe fn as_main_thread_unchecked(&self) -> HostMainThreadHandle<'a> {
        HostMainThreadHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    /// # Safety
    ///
    /// Callers *MUST* ensure this is only called on the audio thread, and that they have exclusive (&mut) access.
    #[inline]
    pub unsafe fn as_audio_thread_unchecked(&self) -> HostAudioThreadHandle<'a> {
        HostAudioThreadHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn use_extension<E: Sized>(&self, extension: &RawExtension<HostExtensionSide, E>) -> &'a E {
        if self.raw != extension.host_ptr() {
            mismatched_instance();
        }

        // SAFETY: the RawExtension type enforces the pointee is valid for as long as the matching
        // instance is still alive.
        unsafe { extension.as_ptr().as_ref() }
    }
}

impl<'a> From<HostSharedHandle<'a>> for HostInfo<'a> {
    #[inline]
    fn from(h: HostSharedHandle<'a>) -> Self {
        h.info()
    }
}

impl<'a> Deref for HostSharedHandle<'a> {
    type Target = HostInfo<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_info()
    }
}

#[repr(transparent)]
pub struct HostMainThreadHandle<'a> {
    raw: NonNull<clap_host>,
    _lifetime: PhantomData<&'a clap_host>,
}

impl<'a> HostMainThreadHandle<'a> {
    #[inline]
    pub fn shared(&self) -> HostSharedHandle<'a> {
        HostSharedHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_shared(&self) -> &HostSharedHandle<'a> {
        // SAFETY: this cast is valid since both types are just a NonNull<clap_host> and repr(transparent)
        unsafe { &*(self as *const Self as *const HostSharedHandle<'a>) }
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        // SAFETY: this type enforces the pointer is valid for 'a
        unsafe { self.raw.as_ref() }
    }
}

impl<'a> From<HostMainThreadHandle<'a>> for HostSharedHandle<'a> {
    #[inline]
    fn from(h: HostMainThreadHandle<'a>) -> Self {
        h.shared()
    }
}

impl<'a> Deref for HostMainThreadHandle<'a> {
    type Target = HostSharedHandle<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_shared()
    }
}

#[repr(transparent)]
pub struct HostAudioThreadHandle<'a> {
    raw: NonNull<clap_host>,
    _lifetime: PhantomData<&'a clap_host>,
}

// SAFETY: this type only exposes the audio-thread-safe (Send) operation of clap_host
unsafe impl<'a> Send for HostAudioThreadHandle<'a> {}

impl<'a> HostAudioThreadHandle<'a> {
    #[inline]
    pub fn shared(&self) -> HostSharedHandle<'a> {
        HostSharedHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        // SAFETY: this type enforces the pointer is valid for 'a
        unsafe { self.raw.as_ref() }
    }

    #[inline]
    pub fn as_shared(&self) -> &HostSharedHandle<'a> {
        // SAFETY: this cast is valid since both types are just a NonNull<clap_host> and repr(transparent)
        unsafe { &*(self as *const Self as *const HostSharedHandle<'a>) }
    }
}

impl<'a> From<HostAudioThreadHandle<'a>> for HostSharedHandle<'a> {
    #[inline]
    fn from(h: HostAudioThreadHandle<'a>) -> Self {
        h.shared()
    }
}

impl<'a> Deref for HostAudioThreadHandle<'a> {
    type Target = HostSharedHandle<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_shared()
    }
}

fn mismatched_instance() -> ! {
    panic!("Given host handle doesn't match the extension pointer it was used on.")
}
