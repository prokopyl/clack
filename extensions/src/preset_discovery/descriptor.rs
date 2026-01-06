use super::*;
use clap_sys::version::{CLAP_VERSION, clap_version};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Provides metadata about a given Provider, such as its ID, name, version, and more.
///
/// All fields of this type as exposed as optional, however the CLAP specification requires the
/// [`id`](ProviderDescriptor::id) and [`name`](ProviderDescriptor::name) fields to be present. It is
/// acceptable for a host to refuse loading a plugin that returns [`None`] for these fields.
///
/// As such, when constructing this type, the  [`id`](ProviderDescriptor::id) and [`name`](ProviderDescriptor::name) fields are
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
/// use clack_extensions::preset_discovery::ProviderDescriptor;
///
/// fn get_descriptor() -> ProviderDescriptor {
///   ProviderDescriptor::new("org.rust-audio.clack.gain-presets", "Clack Gain Example Presets")
/// }
/// ```
#[repr(C)]
#[derive(Clone)]
pub struct ProviderDescriptor {
    clap_version: clap_version,
    id: OwnedCString,
    name: OwnedCString,
    vendor: OwnedCString,
}

impl ProviderDescriptor {
    /// Creates a new provider descriptor, initializing it with the given Provider ID and name.
    ///
    /// See the documentation of the [`id`](ProviderDescriptor::id) and
    /// [`name`](ProviderDescriptor::name) methods for more information about the `id` and `name`
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
        }
        .with_id(id)
        .with_name(name)
    }

    /// Creates a [`ProviderDescriptor`] reference from a pointer to a raw, C-FFI compatible CLAP
    /// descriptor structure.
    ///
    /// # Safety
    ///
    /// All fields must either be null, or point to a valid for reads, null-terminated C. All
    /// must also be valid for reads for the lifetime of the resulting [`ProviderDescriptor`] reference.
    pub const unsafe fn from_raw(raw: &clap_preset_discovery_provider_descriptor) -> &Self {
        // SAFETY: WARNING WARNING WARNING!!!
        // Even when the caller abides to the above safety rules, we MUST be CERTAIN that neither
        // `set` or `Drop` can be called on any of the transmuted fields, as that would be instant UB.
        // Returning a shared reference here is what makes this sound, as both of those operations
        // require mutable references.
        unsafe { &*(raw as *const clap_preset_discovery_provider_descriptor as *const Self) }
    }

    /// Returns the provider descriptor as a reference to the C-FFI compatible CLAP struct.
    #[inline]
    pub fn as_raw(&self) -> &clap_preset_discovery_provider_descriptor {
        // SAFETY: This type is ABI-compatible with clap_preset_discovery_provider_descriptor
        unsafe { &*(self as *const Self as *const clap_preset_discovery_provider_descriptor) }
    }

    /// The unique identifier of a provider.
    ///
    /// This field is **mandatory**, and should not be blank.
    /// If it is found to be [`None`], it is acceptable for the host to refuse to load this plugin.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn id(&self) -> Option<&CStr> {
        self.id.get()
    }

    /// Sets the provider's unique ID.
    ///
    /// See the [`id`](ProviderDescriptor::id) method documentation for more information.
    ///
    /// # Panics
    ///
    /// This function will panic if the given ID is an empty string.
    ///
    /// This function will also panic if the given ID contains NULL-byte characters, which are invalid.
    pub fn with_id(mut self, id: &str) -> Self {
        if id.is_empty() {
            panic!("Provider ID must not be blank!");
        }

        let id = CString::new(id).expect("Invalid Provider ID");
        self.id.set(Some(id));

        self
    }

    /// The display name of this provider.
    ///
    /// This field is **mandatory**, and should not be blank.
    /// If it is found to be [`None`], it is acceptable for the host to refuse to load this plugin.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn name(&self) -> Option<&CStr> {
        self.name.get()
    }

    /// Sets the provider's name.
    ///
    /// See the [`name`](ProviderDescriptor::name) method documentation for more information.
    ///
    /// # Panics
    ///
    /// This function will panic if the given name is an empty string.
    ///
    /// This function will also panic if the given name contains NULL-byte characters, which are invalid.
    pub fn with_name(mut self, name: &str) -> Self {
        if name.is_empty() {
            panic!("Provider name must not be blank!");
        }

        let name = CString::new(name).expect("Invalid Provider name");
        self.name.set(Some(name));

        self
    }

    /// The vendor of the provider.
    ///
    /// This method will return [`None`] if this is either missing or a blank (i.e. empty) string.
    #[inline]
    pub fn vendor(&self) -> Option<&CStr> {
        self.vendor.get()
    }

    /// Sets the provider's vendor name.
    ///
    /// See the [`vendor`](ProviderDescriptor::vendor) method documentation for more information.
    ///
    /// Passing an empty string as the `vendor` parameter will mark it as unset, making
    /// [`vendor`](ProviderDescriptor::vendor) then return `None`.
    ///
    /// # Panics
    ///
    /// This function will also panic if the given vendor name contains NULL-byte characters,
    /// which are invalid.
    pub fn with_vendor(mut self, vendor: &str) -> Self {
        self.vendor.set_str(vendor);
        self
    }
}

const _: () = {
    assert!(
        align_of::<clap_preset_discovery_provider_descriptor>() == align_of::<ProviderDescriptor>()
    );
    assert!(
        size_of::<clap_preset_discovery_provider_descriptor>() == size_of::<ProviderDescriptor>()
    );
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
/// In this case, you **MUST NOT** drop it, or call `set`, which would cause immediate UB.
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

        let string = CString::new(string).expect("Invalid provider descriptor string.");
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
