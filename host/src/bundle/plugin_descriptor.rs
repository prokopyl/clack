use clap_sys::plugin::clap_plugin_descriptor;
use std::ffi::CStr;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct PluginDescriptor<'a> {
    descriptor: &'a clap_plugin_descriptor,
}

unsafe fn cstr_to_str<'a>(ptr: *const std::os::raw::c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        return None;
    }

    // TODO: check for potential malformed data by checking against a maximum size
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

    #[inline]
    pub fn features(&self) -> impl Iterator<Item = &'a CStr> {
        FeaturesIter {
            current: self.descriptor.features as *mut _,
            _lifetime: PhantomData,
        }
    }
}

struct FeaturesIter<'a> {
    current: *mut *const std::os::raw::c_char,
    _lifetime: PhantomData<&'a CStr>,
}

impl<'a> Iterator for FeaturesIter<'a> {
    type Item = &'a CStr;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: list is guaranteed to be null-terminated
        let current = unsafe { self.current.as_ref() }?;
        let cstr = unsafe { CStr::from_ptr(*current) };
        // SAFETY: we just checked the current element was non-null, so there must be another element next
        self.current = unsafe { self.current.add(1) };
        Some(cstr)
    }
}
