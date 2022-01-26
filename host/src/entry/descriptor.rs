use clap_sys::plugin::clap_plugin_descriptor;
use std::ffi::CStr;

pub struct PluginDescriptor<'a> {
    descriptor: &'a clap_plugin_descriptor,
}

unsafe fn cstr_to_str<'a>(ptr: *const ::std::os::raw::c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        return None;
    }

    Some(CStr::from_ptr(ptr))
}

impl<'a> PluginDescriptor<'a> {
    #[inline]
    pub(crate) fn from_raw(descriptor: &'a clap_plugin_descriptor) -> Self {
        Self { descriptor }
    }

    pub fn id(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.id) }
    }

    pub fn name(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.name) }
    }

    pub fn vendor(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.vendor) }
    }

    pub fn url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.url) }
    }

    pub fn manual_url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.manual_url) }
    }

    pub fn support_url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.support_url) }
    }

    pub fn version(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.version) }
    }

    pub fn description(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.description) }
    }

    pub fn keywords_string(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.keywords) }
    }

    pub fn keywords(&self) -> impl Iterator<Item = &'a [u8]> {
        self.keywords_string()
            .unwrap_or_default()
            .to_bytes()
            .split(|b| *b == b';')
    }
}
