use clap_sys::plugin::clap_plugin_descriptor;
use clap_sys::version::CLAP_VERSION;
use std::ffi::CStr;
use std::os::raw::c_char;

#[repr(C)]
pub struct PluginDescriptor(pub(crate) clap_plugin_descriptor);

const EMPTY: &[u8] = b"\0"; // TODO

impl PluginDescriptor {
    pub const fn new(id: &'static [u8]) -> Self {
        PluginDescriptor(clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: check_cstr(id).as_ptr(),
            name: EMPTY.as_ptr() as *const c_char,
            vendor: EMPTY.as_ptr() as *const c_char,
            url: EMPTY.as_ptr() as *const c_char,
            manual_url: EMPTY.as_ptr() as *const c_char,
            version: EMPTY.as_ptr() as *const c_char,
            description: EMPTY.as_ptr() as *const c_char,
            support_url: EMPTY.as_ptr() as *const c_char,
            features: core::ptr::null(), // TODO: check with real features, there seems to be a crash somewhere
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

    // SAFETY: we checked
    unsafe { CStr::from_bytes_with_nul_unchecked(bytes) }
}
