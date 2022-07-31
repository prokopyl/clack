use clap_sys::plugin::clap_plugin_descriptor;
use std::ffi::CStr;
use std::marker::PhantomData;

/// Used to get information about a plugin, like its ID, name,
/// and vendor. The ID is used to instantiate a plugin in
/// [`PluginInstance::new`](crate::instance::PluginInstance::new)
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

#[allow(rustdoc::bare_urls)]
impl<'a> PluginDescriptor<'a> {
    #[inline]
    pub(crate) fn from_raw(descriptor: &'a clap_plugin_descriptor) -> Self {
        Self { descriptor }
    }

    /// Returns the plugin ID, usually in reverse URL format, e.g. "com.u-he.diva"
    pub fn id(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.id) }
    }

    /// Returns the human-readable name, e.g. "Diva"
    pub fn name(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.name) }
    }

    /// Returns the vendor, e.g. "u-he"
    pub fn vendor(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.vendor) }
    }

    /// Returns the url for the plugin, e.g. "https://u-he.com/products/diva/"
    pub fn url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.url) }
    }

    /// Returns the url for the plugin manual,
    /// e.g. "https://dl.u-he.com/manuals/plugins/diva/Diva-user-guide.pdf"
    pub fn manual_url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.manual_url) }
    }

    /// Returns the url for support, e.g. "https://u-he.com/support/"
    pub fn support_url(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.support_url) }
    }

    /// Returns the version number, e.g. "1.4.4"
    pub fn version(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.version) }
    }

    /// Returns a description of the plugin, e.g. "The spirit of analogue"
    pub fn description(&self) -> Option<&'a CStr> {
        unsafe { cstr_to_str(self.descriptor.description) }
    }

    /// Returns a list of keywords that act as feature tags for the plugin.
    /// The host can use these to classify the plugin.
    /// These keywords can be any string, but
    /// [some standard feature keywords are defined here](../../clack_plugin/plugin/descriptor/features/index.html)
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
