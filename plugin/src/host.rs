use clack_common::extensions::{Extension, HostExtension};
use clap_sys::host::clap_host;
use clap_sys::version::clap_version;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct HostInfo<'a> {
    pub inner: &'a clap_host,
}

impl<'a> HostInfo<'a> {
    /// # Safety
    /// Pointer must be valid
    #[inline]
    pub unsafe fn from_raw(raw: *const clap_host) -> Self {
        Self { inner: &*raw }
    }

    #[inline]
    pub fn clap_version(&self) -> clap_version {
        self.inner.clap_version
    }

    pub fn name(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.inner.name) }
            .to_str()
            .expect("Failed to read host name: invalid UTF-8 sequence")
    }

    pub fn vendor(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.inner.vendor) }
            .to_str()
            .expect("Failed to read host vendor: invalid UTF-8 sequence")
    }

    pub fn url(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.inner.url) }
            .to_str()
            .expect("Failed to read host url: invalid UTF-8 sequence")
    }

    pub fn version(&self) -> &'a str {
        unsafe { CStr::from_ptr(self.inner.version) }
            .to_str()
            .expect("Failed to read host version: invalid UTF-8 sequence")
    }

    pub fn get_extension<E: Extension<ExtensionType = HostExtension>>(&self) -> Option<&E> {
        let ptr =
            unsafe { (self.inner.get_extension)(self.inner, E::IDENTIFIER as *const i8) } as *mut _;
        NonNull::new(ptr).map(|p| unsafe { E::from_extension_ptr(p) })
    }

    /// # Safety
    /// Some functions exposed by HostHandle cannot be called until plugin is initialized
    #[inline]
    pub unsafe fn to_handle(self) -> HostHandle<'a> {
        HostHandle { inner: self.inner }
    }
}

#[derive(Copy, Clone)]
pub struct HostHandle<'a> {
    inner: &'a clap_host,
}

impl<'a> HostHandle<'a> {
    #[inline]
    pub fn info(&self) -> HostInfo<'a> {
        HostInfo { inner: self.inner }
    }

    #[inline]
    pub fn as_raw(&self) -> &'a clap_host {
        self.inner
    }

    #[inline]
    pub fn request_restart(&self) {
        // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
        unsafe { (self.inner.request_restart)(self.inner) }
    }

    #[inline]
    pub fn request_process(&self) {
        // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
        unsafe { (self.inner.request_process)(self.inner) }
    }

    #[inline]
    pub fn request_callback(&self) {
        // SAFETY: field is guaranteed to be correct by host. Lifetime is enforced by 'a
        unsafe { (self.inner.request_callback)(self.inner) }
    }

    #[inline]
    pub fn extension<E: Extension<ExtensionType = HostExtension>>(&self) -> Option<&'a E> {
        let id = E::IDENTIFIER;
        let ptr = unsafe { (self.inner.get_extension)(self.inner, id as *const _) as *mut _ };
        unsafe { Some(E::from_extension_ptr(NonNull::new(ptr)?)) }
    }
}

#[derive(Copy, Clone)]
pub struct HostMainThreadHandle<'a> {
    inner: &'a clap_host,
    _non_send: PhantomData<*const clap_host>,
}

impl<'a> HostMainThreadHandle<'a> {
    #[inline]
    pub fn shared(&self) -> HostHandle<'a> {
        HostHandle { inner: self.inner }
    }

    #[inline]
    pub fn extension<E: Extension>(&self) -> Option<&'a E> {
        let id = E::IDENTIFIER;
        let ptr = unsafe { (self.inner.get_extension)(self.inner, id as *const _) as *mut _ };
        unsafe { Some(E::from_extension_ptr(NonNull::new(ptr)?)) }
    }
}
