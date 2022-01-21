use clap_sys::plugin::{clap_plugin_descriptor, CLAP_PLUGIN_AUDIO_EFFECT};
use clap_sys::version::CLAP_VERSION;
use std::ffi::CStr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PluginDescriptor(pub(crate) clap_plugin_descriptor);

const EMPTY: &[u8] = b"\0"; // TODO

impl PluginDescriptor {
    pub const fn new(id: &'static [u8]) -> Self {
        PluginDescriptor(clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: check_cstr(id).as_ptr(),
            name: EMPTY.as_ptr() as *const i8,
            vendor: EMPTY.as_ptr() as *const i8,
            url: EMPTY.as_ptr() as *const i8,
            manual_url: EMPTY.as_ptr() as *const i8,
            version: EMPTY.as_ptr() as *const i8,
            description: EMPTY.as_ptr() as *const i8,
            keywords: EMPTY.as_ptr() as *const i8,
            support_url: EMPTY.as_ptr() as *const i8,
            // FIXME: Why is this u64 but plugin types are i32?
            plugin_type: CLAP_PLUGIN_AUDIO_EFFECT as u64, // TODO
        })
    }

    #[inline]
    pub fn id(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(self.0.id) }
    }
}

const fn check_cstr(bytes: &[u8]) -> &CStr {
    if bytes[bytes.len() - 1] != b'\0' {
        panic!("Invalid C String: string is not null-terminated")
    }
    unsafe { ::core::mem::transmute(bytes) }
}
