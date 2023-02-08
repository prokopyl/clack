use crate::extensions::wrapper::HostWrapper;
use crate::host::{Host, HostExtensions, HostInfo, HostShared};
use clack_common::utils::ClapVersion;
use clap_sys::host::clap_host;
use std::ffi::{c_void, CStr};

pub struct RawHostDescriptor {
    raw: clap_host,
    _host_info: HostInfo,
}

impl RawHostDescriptor {
    pub(crate) fn new<H: for<'h> Host<'h>>(host_info: HostInfo) -> Self {
        let mut raw = clap_host {
            clap_version: ClapVersion::CURRENT.to_raw(),
            host_data: core::ptr::null_mut(),
            name: core::ptr::null_mut(),
            vendor: core::ptr::null_mut(),
            url: core::ptr::null_mut(),
            version: core::ptr::null_mut(),
            get_extension: Some(get_extension::<H>),
            request_restart: Some(request_restart::<H>),
            request_process: Some(request_process::<H>),
            request_callback: Some(request_callback::<H>),
        };

        host_info.write_to_raw(&mut raw);

        Self {
            raw,
            _host_info: host_info,
        }
    }

    #[inline]
    pub fn raw(&self) -> *const clap_host {
        &self.raw
    }

    #[inline]
    pub fn set_wrapper<H: for<'h> Host<'h>>(&mut self, wrapper: &HostWrapper<H>) {
        self.raw.host_data = wrapper as *const _ as *mut _
    }
}

unsafe extern "C" fn get_extension<H: for<'a> Host<'a>>(
    host: *const clap_host,
    identifier: *const std::os::raw::c_char,
) -> *const c_void {
    let identifier = CStr::from_ptr(identifier);
    let mut builder = HostExtensions::new(identifier);

    HostWrapper::<H>::handle(host, |h| {
        H::declare_extensions(&mut builder, h.shared());
        Ok(())
    });
    builder.found()
}

unsafe extern "C" fn request_restart<H: for<'a> Host<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_restart();
        Ok(())
    });
}

unsafe extern "C" fn request_process<H: for<'a> Host<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_process();
        Ok(())
    });
}

unsafe extern "C" fn request_callback<H: for<'a> Host<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_callback();
        Ok(())
    });
}
