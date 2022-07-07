use crate::extensions::HostExtensions;
use crate::host::{HostShared, PluginHoster, SharedHoster};
use crate::wrapper::HostWrapper;
use clack_common::version::ClapVersion;
use clap_sys::host::clap_host;
use selfie::refs::RefType;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct RawHostDescriptor<'a> {
    raw: clap_host,
    _shared: Arc<HostShared>,
    _wrapper: PhantomData<&'a ()>,
}

impl<'a> RawHostDescriptor<'a> {
    pub(crate) fn new<H: for<'h> PluginHoster<'h>>(
        host_shared: Arc<HostShared>,
        wrapper: &'a HostWrapper<H>,
    ) -> Self {
        let mut raw = clap_host {
            clap_version: ClapVersion::CURRENT.to_raw(),
            host_data: wrapper as *const _ as *mut _,
            name: core::ptr::null_mut(),
            vendor: core::ptr::null_mut(),
            url: core::ptr::null_mut(),
            version: core::ptr::null_mut(),
            get_extension: get_extension::<H>,
            request_restart: request_restart::<H>,
            request_process: request_process::<H>,
            request_callback: request_callback::<H>,
        };

        host_shared.info().write_to_raw(&mut raw);

        Self {
            raw,
            _shared: host_shared,
            _wrapper: PhantomData,
        }
    }

    #[inline]
    pub fn raw(&self) -> &clap_host {
        &self.raw
    }
}

pub struct RawHostDescriptorRef;

impl<'a> RefType<'a> for RawHostDescriptorRef {
    type Ref = RawHostDescriptor<'a>;
}

unsafe extern "C" fn get_extension<H: for<'a> PluginHoster<'a>>(
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

unsafe extern "C" fn request_restart<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_restart();
        Ok(())
    });
}

unsafe extern "C" fn request_process<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_process();
        Ok(())
    });
}

unsafe extern "C" fn request_callback<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_callback();
        Ok(())
    });
}
