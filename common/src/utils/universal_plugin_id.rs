use clap_sys::universal_plugin_id::clap_universal_plugin_id;
use core::ffi::CStr;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UniversalPluginID<'a> {
    pub abi: &'a CStr,
    pub id: &'a CStr,
}

impl<'a> UniversalPluginID<'a> {
    pub const unsafe fn from_raw_ptr(ptr: *const clap_universal_plugin_id) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: TODO
        let plugin_id = unsafe { ptr.read() };
        Self::from_raw(plugin_id)
    }

    pub const unsafe fn from_raw(raw: clap_universal_plugin_id) -> Option<Self> {
        Some(Self {
            abi: if raw.abi.is_null() {
                return None;
            } else {
                // SAFETY: TODO
                unsafe { CStr::from_ptr(raw.abi) }
            },
            id: if raw.id.is_null() {
                return None;
            } else {
                // SAFETY: TODO
                unsafe { CStr::from_ptr(raw.id) }
            },
        })
    }

    #[inline]
    pub const fn clap(id: &'a CStr) -> Self {
        Self { abi: c"clap", id }
    }

    #[inline]
    pub const fn to_raw(&self) -> clap_universal_plugin_id {
        clap_universal_plugin_id {
            id: self.id.as_ptr(),
            abi: self.abi.as_ptr(),
        }
    }
}
