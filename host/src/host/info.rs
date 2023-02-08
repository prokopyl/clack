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

/// Human-readable information and description about the host.
///
/// This information is passed to plugins at instantiation time by [`PluginInstance::new`](crate::prelude::PluginInstance::new).
///
/// See the [`new`](HostInfo::new) method's documentation for an example of how to instantiate it.
#[derive(Debug, Clone)]
pub struct HostInfo {
    inner: Arc<HostInfoInner>,
}

impl HostInfo {
    /// Creates a new host information container from its components.
    ///
    /// * `name`: The name of the host. Example: `"Bitwig Studio"`
    /// * `vendor`: The software vendor of the host. Example: `"Bitwig GmbH"`
    /// * `url`: Example: `"https://bitwig.com"`
    /// * `version`: The version string of the host. Example: `"4.3.2"`
    ///
    /// All parameters are copied into new string buffers.
    ///
    /// # Errors
    ///
    /// All parameters must not contain the null (`\0`) character. If any of them do, a [`NulError`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_host::host::HostInfo;
    ///
    /// let info = HostInfo::new("Bitwig Studio", "Bitwig GmbH", "https://bitwig.com", "4.3.2");
    /// ```
    ///
    pub fn new(name: &str, vendor: &str, url: &str, version: &str) -> Result<Self, NulError> {
        Ok(Self::new_from_cstring(
            CString::new(name)?,
            CString::new(vendor)?,
            CString::new(url)?,
            CString::new(version)?,
        ))
    }

    /// An infallible version of [`new`](HostInfo::new).
    ///
    /// This method takes ownership of preexisting [`CString`] buffers, and therefore cannot fail
    /// from null byte errors, and performs less allocations compared to [`new`](HostInfo::new).
    ///
    /// See the documentation for [`new`](HostInfo::new) for more information about the arguments.
    pub fn new_from_cstring(
        name: CString,
        vendor: CString,
        url: CString,
        version: CString,
    ) -> Self {
        Self {
            inner: Arc::new(HostInfoInner {
                name: Pin::new(name.into_boxed_c_str()),
                vendor: Pin::new(vendor.into_boxed_c_str()),
                url: Pin::new(url.into_boxed_c_str()),
                version: Pin::new(version.into_boxed_c_str()),
            }),
        }
    }

    pub(crate) fn write_to_raw(&self, host: &mut clap_host) {
        host.name = self.inner.name.as_ptr();
        host.vendor = self.inner.vendor.as_ptr();
        host.url = self.inner.url.as_ptr();
        host.version = self.inner.version.as_ptr();
    }
}
