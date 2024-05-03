use clap_sys::plugin::clap_plugin_descriptor;
use std::ffi::CStr;
use std::marker::PhantomData;

/// Various textual information about a plugin.
///
/// The information contained in this type can be exposed to a host's user in e.g. a list, to
/// select which plugin to load.
///
/// All fields of this type as exposed as optional, however the CLAP specification requires the
/// [`id`](PluginDescriptor::id) and [`name`](PluginDescriptor::name) fields to be present. It is
/// acceptable for a host to refuse loading a plugin that returns [`None`] for these fields.
///
/// Note that all the fields of this type as exposed as the CLAP-native [`CStr`], as they are not
/// required by the CLAP spec to be UTF-8 compliant, only exposing byte slices. Hosts are left to
/// interpret non-UTF-8 data best they can.
///
/// See [`PluginFactory::plugin_descriptors`](super::PluginFactory::plugin_descriptors), which
/// returns all the plugin descriptors exposed by a [plugin bundle](crate::bundle).
#[derive(Copy, Clone)]
pub struct PluginDescriptor<'a> {
    descriptor: &'a clap_plugin_descriptor,
}

/// # Safety
///
/// Same as [`CStr::from_ptr`], except the given pointer *can* be null.
unsafe fn cstr_to_str<'a>(ptr: *const std::os::raw::c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        return None;
    }

    let string = CStr::from_ptr(ptr);

    if string.to_bytes().is_empty() {
        None
    } else {
        Some(string)
    }
}

impl<'a> PluginDescriptor<'a> {
    /// # Safety
    /// The user must ensure the provided descriptor is valid, including all of its pointers.
    #[inline]
    pub(crate) unsafe fn from_raw(descriptor: &'a clap_plugin_descriptor) -> Self {
        Self { descriptor }
    }

    /// An arbitrary string identifier that is unique to this plugin.
    ///
    /// Plugins are encouraged to use a reverse-URI for this, e.g. `com.u-he.diva`, but this is not
    /// a strict requirement.
    ///
    /// This is required to be globally-unique, and is therefore safe and intended for host to use
    /// to uniquely refer to this plugin across different saves and machines.
    ///
    /// This is as exposed as optional, however the CLAP specification requires it to be
    /// present. It is acceptable for a host to refuse loading a plugin that returns [`None`] here.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"com.u-he.diva", descriptor.id().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn id(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.id) }
    }

    /// The user-facing display name of this plugin.
    ///
    /// This is as exposed as optional, however the CLAP specification requires it to be
    /// present. It is acceptable for a host to refuse loading a plugin that returns [`None`] here.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"Diva", descriptor.name().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn name(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.name) }
    }

    /// The vendor of this plugin.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"u-he", descriptor.vendor().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn vendor(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.vendor) }
    }

    /// The URL of this plugin's homepage.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"https://u-he.com/products/diva/", descriptor.url().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn url(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.url) }
    }

    /// The URL of this plugin's user's manual.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(
    ///     b"https://dl.u-he.com/manuals/plugins/diva/Diva-user-guide.pdf",
    ///     descriptor.manual_url().unwrap().to_bytes()
    /// );
    /// # }
    /// ```
    pub fn manual_url(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.manual_url) }
    }

    /// The URL of this plugin's support page.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"https://u-he.com/support/", descriptor.support_url().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn support_url(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.support_url) }
    }

    /// The version of this plugin.
    ///
    /// While Semver-compatible version strings are recommended, this field can contain any arbitrary
    /// string.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"1.4.4", descriptor.version().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn version(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.version) }
    }

    /// A short description of this plugin.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// assert_eq!(b"The spirit of analogue", descriptor.description().unwrap().to_bytes());
    /// # }
    /// ```
    pub fn description(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the string pointer is valid
        unsafe { cstr_to_str(self.descriptor.description) }
    }

    /// An iterator over an arbitrary list of tags, that can be used by hosts to classify this plugin.
    ///
    /// # Example
    /// ```
    /// use clack_host::factory::PluginDescriptor;
    ///
    /// # fn x(descriptor: &PluginDescriptor) {
    /// let descriptor: &PluginDescriptor = /* ... */
    /// # unreachable!();
    /// let features: Vec<_> = descriptor.features().map(|s| s.to_bytes()).collect();
    /// assert_eq!([b"instrument".as_slice(), b"synthesizer".as_slice(), b"stereo".as_slice()].as_slice(), &features);
    /// # }
    /// ```
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
