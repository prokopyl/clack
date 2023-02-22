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
            current: self.descriptor.features,
            _lifetime: PhantomData,
        }
    }
}

struct FeaturesIter<'a> {
    current: *const *const std::os::raw::c_char,
    _lifetime: PhantomData<&'a CStr>,
}

impl<'a> Iterator for FeaturesIter<'a> {
    type Item = &'a CStr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        // SAFETY: we just null-checked the list pointer above.
        let current = unsafe { self.current.as_ref() }?;
        if current.is_null() {
            return None;
        }

        // SAFETY: we just checked the current element was non-null
        let cstr = unsafe { CStr::from_ptr(*current) };
        // SAFETY: The current element is non-null, so there must be another element next
        self.current = unsafe { self.current.add(1) };
        Some(cstr)
    }
}
