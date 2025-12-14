use clap_sys::plugin::clap_plugin_descriptor;
use clap_sys::version::{CLAP_VERSION, clap_version};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;

/// Provides metadata about a given Plugin, such as its ID, name, version, and more.
///
/// All fields of this type as exposed as optional, however the CLAP specification requires the
/// [`id`](PluginDescriptor::id) and [`name`](PluginDescriptor::name) fields to be present. It is
/// acceptable for a host to refuse loading a plugin that returns [`None`] for these fields.
///
/// As such, when constructing this type, the  [`id`](PluginDescriptor::id) and [`name`](PluginDescriptor::name) fields are
/// mandatory, and should not be blank. All the other fields are completely optional.
///
/// Note that all the read accessors of this type are exposed as the CLAP-native [`CStr`], as they are not
/// required by the CLAP spec to be UTF-8 compliant, only exposing byte slices. Hosts are left to
/// interpret non-UTF-8 data best they can.
///
/// The write accessors on this type take [string](str) references for convenience reasons, but they
/// will still internally convert them into null-terminated C strings, and panic if that conversion fails.
///
/// See the documentation each accessor method to learn about the available metadata.
///
/// # Example
///
/// ```
/// use clack_common::plugin::PluginDescriptor;
///
/// fn get_descriptor() -> PluginDescriptor {
///   use clack_common::plugin::features::*;
///
///   PluginDescriptor::new("org.rust-audio.clack.gain", "Clack Gain Example")
///     .with_description("A simple gain plugin example!")
///     .with_version("0.1.0")
///     .with_features([AUDIO_EFFECT, STEREO])
/// }
/// ```
#[repr(C)]
#[derive(Clone)]
pub struct PluginDescriptor {
    clap_version: clap_version,
    id: OwnedCString,
    name: OwnedCString,
    vendor: OwnedCString,
    url: OwnedCString,
    manual_url: OwnedCString,
    support_url: OwnedCString,
    version: OwnedCString,
    description: OwnedCString,
    features: OwnedCStringArray,
}

impl PluginDescriptor {
    /// Creates a new plugin descriptor, initializing it with the given Plugin ID and name.
    ///
    /// See the documentation of the [`id`](PluginDescriptor::id) and
    /// [`name`](PluginDescriptor::name) methods for more information about the `id` and `name`
    /// parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if either the given ID or name are empty strings.
    ///
    /// This function will also panic if either the given ID or name contain invalid NULL-byte
    /// characters, which are invalid.
    #[inline]
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            clap_version: CLAP_VERSION,
            id: OwnedCString::empty(),
            name: OwnedCString::empty(),
            vendor: OwnedCString::empty(),
            url: OwnedCString::empty(),
            manual_url: OwnedCString::empty(),
            support_url: OwnedCString::empty(),
            version: OwnedCString::empty(),
            description: OwnedCString::empty(),
            features: OwnedCStringArray::empty(),
        }
        .with_id(id)
        .with_name(name)
    }

    /// Creates a [`PluginDescriptor`] reference from a pointer to a raw, C-FFI compatible CLAP
    /// descriptor structure.
    ///
    /// # Safety
    ///
    /// All fields must either be null, or point to a valid for reads, null-terminated C
    /// string (or for `features`, a null-terminated array of null-terminated C strings), which all
    /// must also be valid for reads for the lifetime of the resulting [`PluginDescriptor`] reference.
    pub const unsafe fn from_raw(raw: &clap_plugin_descriptor) -> &Self {
        // SAFETY: WARNING WARNING WARNING!!!
        // Even when the caller abides to the above safety rules, we MUST be CERTAIN that neither
        // `set` or `Drop` can be called on any of the transmuted fields, as that would be instant UB.
        // Returning a shared reference here is what makes this sound, as both of those operations
        // require mutable references.
        unsafe { &*(raw as *const clap_plugin_descriptor as *const Self) }
    }

    /// Returns the plugin descriptor as a reference to the C-FFI compatible CLAP struct.
    #[inline]
    pub fn as_raw(&self) -> &clap_plugin_descriptor {
        // SAFETY: This type is ABI-compatible with clap_plugin_descriptor
        unsafe { &*(self as *const Self as *const clap_plugin_descriptor) }
    }

    /// The unique identifier of a plugin.
    ///
    /// This field is **mandatory**, and should not be blank.
    /// If it is found to be [`None`], it is acceptable for the host to refuse to load this plugin.
    ///
    /// This identifier should be as globally-unique as possible to any users that might load this
    /// plugin, as this is the key hosts will use to differentiate between different plugins.
    ///
    /// Example: `com.u-he.diva`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn id(&self) -> Option<&CStr> {
        self.id.get()
    }

    /// Sets the plugin's unique ID.
    ///
    /// See the [`id`](PluginDescriptor::id) method documentation for more information.
    ///
    /// # Panics
    ///
    /// This function will panic if the given ID is an empty string.
    ///
    /// This function will also panic if the given ID contains NULL-byte characters, which are invalid.
    pub fn with_id(mut self, id: &str) -> Self {
        if id.is_empty() {
            panic!("Plugin ID must not be blank!");
        }

        let id = CString::new(id).expect("Invalid Plugin ID");
        self.id.set(Some(id));

        self
    }

    /// The user-facing display name of this plugin.
    ///
    /// This field is **mandatory**, and should not be blank.
    /// If it is found to be [`None`], it is acceptable for the host to refuse to load this plugin.
    ///
    /// This name will be displayed in plugin lists and selectors, and will be the main way users
    /// will find and differentiate the plugin.
    ///
    /// Example: `Diva`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn name(&self) -> Option<&CStr> {
        self.name.get()
    }

    /// Sets the plugin's name.
    ///
    /// See the [`name`](PluginDescriptor::name) method documentation for more information.
    ///
    /// # Panics
    ///
    /// This function will panic if the given name is an empty string.
    ///
    /// This function will also panic if the given name contains NULL-byte characters, which are invalid.
    pub fn with_name(mut self, name: &str) -> Self {
        if name.is_empty() {
            panic!("Plugin name must not be blank!");
        }

        let name = CString::new(name).expect("Invalid Plugin name");
        self.name.set(Some(name));

        self
    }

    /// The vendor of the plugin.
    ///
    /// Example: `u-he`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn vendor(&self) -> Option<&CStr> {
        self.vendor.get()
    }

    /// Sets the plugin's vendor name.
    ///
    /// See the [`vendor`](PluginDescriptor::vendor) method documentation for more information.
    ///
    /// Passing an empty string as the `vendor` parameter will mark it as unset, making
    /// [`vendor`](PluginDescriptor::vendor) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given vendor name contains NULL-byte characters,
    /// which are invalid.
    pub fn with_vendor(mut self, vendor: &str) -> Self {
        self.vendor.set_str(vendor);
        self
    }

    /// The URL of this plugin's homepage.
    ///
    /// Example: `https://u-he.com/products/diva/`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn url(&self) -> Option<&CStr> {
        self.url.get()
    }

    /// Sets the plugin's homepage's URL.
    ///
    /// See the [`url`](PluginDescriptor::url) method documentation for more information.
    ///
    /// Passing an empty string as the `url` parameter will mark it as unset, making
    /// [`url`](PluginDescriptor::url) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given URL contains NULL-byte characters,
    /// which are invalid.
    pub fn with_url(mut self, url: &str) -> Self {
        self.url.set_str(url);
        self
    }

    /// The URL of this plugin's user's manual.
    ///
    /// Example: `https://dl.u-he.com/manuals/plugins/diva/Diva-user-guide.pdf`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn manual_url(&self) -> Option<&CStr> {
        self.manual_url.get()
    }

    /// Sets the plugin's manual's URL.
    ///
    /// See the [`url`](PluginDescriptor::url) method documentation for more information.
    ///
    /// Passing an empty string as the `url` parameter will mark it as unset, making
    /// [`url`](PluginDescriptor::url) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given URL contains NULL-byte characters,
    /// which are invalid.
    pub fn with_manual_url(mut self, manual_url: &str) -> Self {
        self.manual_url.set_str(manual_url);
        self
    }

    /// The URL of this plugin's support page.
    ///
    /// Example: `https://u-he.com/support/`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn support_url(&self) -> Option<&CStr> {
        self.support_url.get()
    }

    /// Sets the plugin's support URL.
    ///
    /// See the [`support_url`](PluginDescriptor::support_url) method documentation for more information.
    ///
    /// Passing an empty string as the `support_url` parameter will mark it as unset, making
    /// [`support_url`](PluginDescriptor::support_url) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given URL contains NULL-byte characters,
    /// which are invalid.
    pub fn with_support_url(mut self, support_url: &str) -> Self {
        self.support_url.set_str(support_url);
        self
    }

    /// The version string of this plugin.
    ///
    /// While Semver-compatible version strings are preferred, this field can contain any arbitrary
    /// string.
    ///
    /// Example: `1.4.4`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn version(&self) -> Option<&CStr> {
        self.version.get()
    }

    /// Sets the plugin's version string.
    ///
    /// See the [`version`](PluginDescriptor::version) method documentation for more information.
    ///
    /// Passing an empty string as the `version` parameter will mark it as unset, making
    /// [`version`](PluginDescriptor::version) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given version string contains NULL-byte characters,
    /// which are invalid.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version.set_str(version);
        self
    }

    /// A short description of this plugin.
    ///
    /// Example: `The spirit of analogue`.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn description(&self) -> Option<&CStr> {
        self.description.get()
    }

    /// Sets the plugin's description.
    ///
    /// See the [`description`](PluginDescriptor::description) method documentation for more information.
    ///
    /// Passing an empty string as the `description` parameter will mark it as unset, making
    /// [`description`](PluginDescriptor::description) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given description contains NULL-byte characters,
    /// which are invalid.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description.set_str(description);
        self
    }

    /// An arbitrary list of tags that can be used by hosts to classify this plugin.
    ///
    /// For some standard features, see the constants in the [`features`](super::features) module.
    ///
    /// Example: `"instrument", "synthesizer", "stereo"`.
    #[inline]
    pub fn features(&self) -> FeaturesIter<'_> {
        self.features.get()
    }

    /// Sets the plugin's feature list.
    ///
    /// See the [`features`](PluginDescriptor::features) method documentation for more information.
    pub fn with_features<'a>(mut self, features: impl IntoIterator<Item = &'a CStr>) -> Self {
        self.features.set(features);

        self
    }
}

const _: () = {
    assert!(align_of::<clap_plugin_descriptor>() == align_of::<PluginDescriptor>());
    assert!(size_of::<clap_plugin_descriptor>() == size_of::<PluginDescriptor>());
};

static EMPTY: &CStr = c"";

/// # Safety Invariants
///
/// This type's inner pointer can either be :
///
/// * null;
/// * pointing to `EMPTY`;
/// * pointing to a string created by CString::into_raw.
///
/// This type alone cannot set its pointer to any other kind of value.
///
/// # Transmuting
/// This type is `#[repr(C)]` and so *can* be transmuted from a raw `*const c_char`, in which case
/// it may hold an arbitrary pointer.
///
/// In this case, you **MUST NOT** drop it, or to call `set`, which would cause immediate UB.
///
/// `get` may be called, but **only** if the pointer points to a valid C String (nul-terminated), and
/// the pointer's value is not changed for the lifetime of this type.
///
/// In any case, this type can always safely be transmuted *to* a raw `*const c_char`.
#[repr(C)]
struct OwnedCString(*const c_char);

// SAFETY: OwnedCString is fully self-contained, the pointers refer to data owned by it.
unsafe impl Send for OwnedCString {}

// SAFETY: OwnedCString does not have any interior mutability.
unsafe impl Sync for OwnedCString {}

impl OwnedCString {
    #[inline]
    pub fn new(string: Option<CString>) -> Self {
        let mut new = Self::empty();
        new.set(string);
        new
    }

    #[inline]
    fn into_raw(self) -> *const c_char {
        let s = ManuallyDrop::new(self);
        s.0
    }

    /// # Safety
    ///
    /// This pointer MUST follow this type's safety invariants.
    ///
    /// See [`OwnedCString`].
    #[inline]
    unsafe fn from_raw(ptr: *const c_char) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(EMPTY.as_ptr())
    }

    #[inline]
    pub fn get(&self) -> Option<&CStr> {
        if Self::is_allocated(self.0) {
            // SAFETY: From our own invariants
            Some(unsafe { CStr::from_ptr(self.0) })
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, new: Option<CString>) {
        // We do this just in case *something* panics, in which case this instance remains valid and worst case we just leak some memory.
        let old_ptr = core::mem::replace(&mut self.0, EMPTY.as_ptr());

        // SAFETY: per our own invariants, this pointer is either null, EMPTY, or from to_raw.
        unsafe { Self::deallocate(old_ptr) };

        // If the string is not empty, we overwrite our EMPTY pointer with an owned one
        if let Some(new) = new {
            if !new.is_empty() {
                // Note: this allocates, which implies it may panic. In that case we're good, because
                // our previous inner value of EMPTY is completely valid.
                self.0 = new.into_raw();
            }
        }

        // If the string is empty, then we are already set to EMPTY, so we have nothing to do.
    }

    #[inline]
    pub fn set_str(&mut self, string: &str) {
        if string.is_empty() {
            self.set(None);
            return;
        }

        let string = CString::new(string).expect("Invalid plugin descriptor string.");
        self.set(Some(string));
    }

    #[inline]
    fn is_allocated(ptr: *const c_char) -> bool {
        !ptr.is_null() && ptr != EMPTY.as_ptr()
    }

    /// # Safety
    ///
    /// This must ONLY be called on pointers that are either null, EMPTY, or from [`Self::allocate`].
    #[inline]
    unsafe fn deallocate(old_ptr: *const c_char) {
        if Self::is_allocated(old_ptr) {
            // SAFETY: From our own invariants, if it's not null or EMPTY, then it's from into_raw.
            let _ = unsafe { CString::from_raw(old_ptr.cast_mut()) };
        }
    }
}

impl Clone for OwnedCString {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.get().map(|s| s.to_owned()))
    }
}

impl Drop for OwnedCString {
    #[inline]
    fn drop(&mut self) {
        // Just in case.
        let old_ptr = core::mem::replace(&mut self.0, EMPTY.as_ptr());

        // SAFETY: per our own invariants, this pointer is either null, EMPTY, or from to_raw.
        unsafe { Self::deallocate(old_ptr) };
    }
}

/// The same invariants as for [`OwnedCString`], but for an array.
#[repr(C)]
struct OwnedCStringArray(*const *const c_char);

// SAFETY: OwnedCStringArray is fully self-contained, the pointers refer to data owned by it.
unsafe impl Send for OwnedCStringArray {}

// SAFETY: OwnedCStringArray does not have any interior mutability.
unsafe impl Sync for OwnedCStringArray {}

// Technically this doesn't need to be OwnedCString, it just needs to be (ABI-compatible with)
// any NULL pointer that's Send + Sync.
// I could have made a dedicated wrapper but OwnedCString::empty() fits the bill just as well :)
static EMPTY_FEATURES: &[OwnedCString; 1] = &[OwnedCString::empty()];

impl OwnedCStringArray {
    #[inline]
    fn is_allocated(ptr: *const *const c_char) -> bool {
        !ptr.is_null() && ptr != empty_features()
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(empty_features())
    }

    #[inline]
    pub const fn get(&self) -> FeaturesIter<'_> {
        FeaturesIter {
            ptr: CArrayIter(self.0),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn set<'a>(&mut self, features: impl IntoIterator<Item = &'a CStr>) {
        // We do this just in case *something* panics, in which case this instance remains valid and worst case we just leak some memory.
        let old_ptr = core::mem::replace(&mut self.0, empty_features());

        // SAFETY: per our own invariants, this pointer is either null, EMPTY, or from to_raw.
        unsafe { Self::deallocate(old_ptr) };

        // It is safe to transmute OwnedCString to *const c_char.
        self.0 = Self::allocate(features).cast::<*const c_char>();
    }

    fn allocate<'a>(features: impl IntoIterator<Item = &'a CStr>) -> *const OwnedCString {
        let mut strings = Vec::with_capacity(1);

        for feature in features {
            strings.push(OwnedCString::new(Some(feature.to_owned())).into_raw());
        }

        if strings.is_empty() {
            return EMPTY_FEATURES.as_ptr().cast_mut();
        }

        strings.push(core::ptr::null());

        let strings = strings.into_boxed_slice();
        let strings = Box::into_raw(strings);

        strings.cast()
    }

    /// # Safety
    ///
    /// This must ONLY be called on pointers that are either null, EMPTY_FEATURES, or from [`Self::allocate`].
    unsafe fn deallocate(ptr: *const *const c_char) {
        if !Self::is_allocated(ptr) {
            return;
        }

        // First: deallocate *everything* in here, while tracking the length of the array.
        let mut len = 1; // Include the null terminator.

        // Note: if any of these calls panics, then we simply leak the vec.
        // SAFETY: per our own invariants, this is a null-terminated array that we own (see allocate).
        for str in unsafe { CArrayIter::new(ptr) } {
            // SAFETY: made by OwnedCString::into_raw
            let _ = unsafe { OwnedCString::from_raw(str) };
            len += 1;
        }

        // Then, reconstruct the Boxed slice and deallocate that.
        // SAFETY: this was made from Box::into_raw in allocate.
        let _ = unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr.cast_mut(), len)) };
    }
}

impl Clone for OwnedCStringArray {
    fn clone(&self) -> Self {
        let mut new = Self::empty();
        new.set(self.get());
        new
    }
}

impl Drop for OwnedCStringArray {
    #[inline]
    fn drop(&mut self) {
        // Just in case.
        let old_ptr = core::mem::replace(&mut self.0, empty_features());

        // SAFETY: per our own invariants, this pointer is either null, EMPTY, or from to_raw.
        unsafe { Self::deallocate(old_ptr) };
    }
}

const fn empty_features() -> *const *const c_char {
    EMPTY_FEATURES.as_ptr().cast()
}

pub struct FeaturesIter<'a> {
    ptr: CArrayIter,
    _marker: PhantomData<&'a OwnedCStringArray>,
}

impl<'a> Iterator for FeaturesIter<'a> {
    type Item = &'a CStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.ptr.next()?;

        if next.is_null() {
            return None;
        }

        // SAFETY: upheld by caller to be a valid C string for 'a
        Some(unsafe { CStr::from_ptr(next) })
    }
}

struct CArrayIter(*const *const c_char);

impl CArrayIter {
    /// # Safety
    ///
    /// This must either be NULL, point to a null-terminated C array
    #[inline]
    unsafe fn new(ptr: *const *const c_char) -> Self {
        Self(ptr)
    }
}

impl Iterator for CArrayIter {
    type Item = *const c_char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: Upheld by caller that the array this points to is null-terminated
        let current = *unsafe { self.0.as_ref() }?;

        if current.is_null() {
            return None;
        }

        self.0 = self.0.wrapping_add(1);
        Some(current)
    }
}
