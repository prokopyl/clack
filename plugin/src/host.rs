use clack_common::extensions::{Extension, HostExtensionSide};
use clap_sys::host::clap_host;
use clap_sys::version::clap_version;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct HostInfo<'a> {
    raw: *const clap_host,
    _lifetime: PhantomData<&'a clap_host>,
}

impl<'a> HostInfo<'a> {
    /// # Safety
    /// Pointer must be valid
    #[inline]
    pub unsafe fn from_raw(raw: *const clap_host) -> Self {
        Self {
            raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn clap_version(&self) -> clap_version {
        self.as_raw().clap_version
    }

    pub fn name(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.as_raw().name) }
            .to_str()
            .expect("Failed to read host name: invalid UTF-8 sequence")
    }

    pub fn vendor(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.as_raw().vendor) }
            .to_str()
            .expect("Failed to read host vendor: invalid UTF-8 sequence")
    }

    pub fn url(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.as_raw().url) }
            .to_str()
            .expect("Failed to read host url: invalid UTF-8 sequence")
    }

    pub fn version(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.as_raw().version) }
            .to_str()
            .expect("Failed to read host version: invalid UTF-8 sequence")
    }

    pub fn get_extension<E: Extension<ExtensionSide = HostExtensionSide>>(&self) -> Option<&'a E> {
        let ext =
            unsafe { (self.as_raw().get_extension?)(self.raw, E::IDENTIFIER.as_ptr()) } as *mut _;
        NonNull::new(ext).map(|p| unsafe { E::from_extension_ptr(p) })
    }

    /// # Safety
    /// Some functions exposed by HostHandle cannot be called until plugin is initialized
    #[inline]
    pub(crate) unsafe fn to_handle(self) -> HostHandle<'a> {
        HostHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        unsafe { &*self.raw }
    }
}

#[derive(Copy, Clone)]
pub struct HostHandle<'a> {
    raw: *const clap_host,
    _lifetime: PhantomData<&'a clap_host>,
}

unsafe impl<'a> Send for HostHandle<'a> {}
unsafe impl<'a> Sync for HostHandle<'a> {}

impl<'a> HostHandle<'a> {
    #[inline]
    pub fn info(&self) -> HostInfo<'a> {
        HostInfo {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *const clap_host {
        self.raw
    }

    #[inline]
    pub fn request_restart(&self) {
        if let Some(request_restart) = unsafe { (*self.as_raw()).request_restart } {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_restart(self.raw) }
        }
    }

    #[inline]
    pub fn request_process(&self) {
        if let Some(request_process) = unsafe { (*self.as_raw()).request_process } {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_process(self.raw) }
        }
    }

    #[inline]
    pub fn request_callback(&self) {
        if let Some(request_callback) = unsafe { (*self.as_raw()).request_callback } {
            // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
            unsafe { request_callback(self.raw) }
        }
    }

    #[inline]
    pub fn extension<E: Extension<ExtensionSide = HostExtensionSide>>(&self) -> Option<&'a E> {
        self.info().get_extension()
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
}

impl<'a> From<HostHandle<'a>> for HostInfo<'a> {
    #[inline]
    fn from(h: HostHandle<'a>) -> Self {
        h.info()
    }
}

#[derive(Copy, Clone)]
pub struct HostMainThreadHandle<'a> {
    raw: *const clap_host,
    _lifetime: PhantomData<&'a clap_host>,
}

impl<'a> HostMainThreadHandle<'a> {
    #[inline]
    pub fn shared(&self) -> HostHandle<'a> {
        HostHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn extension<E: Extension<ExtensionSide = HostExtensionSide>>(&self) -> Option<&'a E> {
        self.shared().extension()
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        unsafe { &*self.raw }
    }
}

impl<'a> From<HostMainThreadHandle<'a>> for HostHandle<'a> {
    #[inline]
    fn from(h: HostMainThreadHandle<'a>) -> Self {
        h.shared()
    }
}

#[derive(Copy, Clone)]
pub struct HostAudioThreadHandle<'a> {
    raw: *const clap_host,
    _lifetime: PhantomData<&'a clap_host>,
}

unsafe impl<'a> Send for HostAudioThreadHandle<'a> {}

impl<'a> HostAudioThreadHandle<'a> {
    #[inline]
    pub fn shared(&self) -> HostHandle<'a> {
        HostHandle {
            raw: self.raw,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn extension<E: Extension<ExtensionSide = HostExtensionSide>>(&self) -> Option<&'a E> {
        self.shared().extension()
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        unsafe { &*self.raw }
    }
}

impl<'a> From<HostAudioThreadHandle<'a>> for HostHandle<'a> {
    #[inline]
    fn from(h: HostAudioThreadHandle<'a>) -> Self {
        h.shared()
    }
}
