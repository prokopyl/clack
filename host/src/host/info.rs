use clap_sys::host::clap_host;
use std::ffi::{CStr, CString, NulError};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct HostInfoInner {
    name: Pin<Box<CStr>>,
    vendor: Pin<Box<CStr>>,
    url: Pin<Box<CStr>>,
    version: Pin<Box<CStr>>,
}

#[derive(Debug, Clone)]
pub struct HostInfo {
    inner: Arc<HostInfoInner>,
}

fn to_pin_cstr(str: &str) -> Result<Pin<Box<CStr>>, NulError> {
    Ok(Pin::new(CString::new(str)?.into_boxed_c_str()))
}

impl HostInfo {
    pub fn new(name: &str, vendor: &str, url: &str, version: &str) -> Result<Self, NulError> {
        Ok(Self {
            inner: Arc::new(HostInfoInner {
                name: to_pin_cstr(name)?,
                vendor: to_pin_cstr(vendor)?,
                url: to_pin_cstr(url)?,
                version: to_pin_cstr(version)?,
            }),
        })
    }

    pub(crate) fn write_to_raw(&self, host: &mut clap_host) {
        host.name = self.inner.name.as_ptr();
        host.vendor = self.inner.vendor.as_ptr();
        host.url = self.inner.url.as_ptr();
        host.version = self.inner.version.as_ptr();
    }
}
