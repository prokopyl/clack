use clap_sys::plugin::clap_plugin_descriptor;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::pin::Pin;

/// Represents a type that can provide metadata about a given Plugin, such as its ID, name, version,
/// and more.
///
/// Note only the [`id`](PluginDescriptor::id) and [`name`](PluginDescriptor::name) fields are
/// mandatory, and should not be blank. All the other fields are optional.
///
/// See the documentation each accessor method to learn about the available metadata.
///
/// # Example
///
/// ```
/// use clack_plugin::prelude::PluginDescriptor;
///
/// fn get_descriptor() -> PluginDescriptor {
///   use clack_plugin::plugin::features::*;
///
///   PluginDescriptor::new("org.rust-audio.clack.gain", "Clack Gain Example")
///     .with_description("A simple gain plugin example!")
///     .with_version("0.1.0")
///     .with_features([AUDIO_EFFECT, STEREO])
/// }
/// ```
pub struct PluginDescriptor {
    id: Pin<Box<CStr>>,
    name: Pin<Box<CStr>>,

    vendor: Option<Pin<Box<CStr>>>,
    url: Option<Pin<Box<CStr>>>,
    manual_url: Option<Pin<Box<CStr>>>,
    support_url: Option<Pin<Box<CStr>>>,
    version: Option<Pin<Box<CStr>>>,
    description: Option<Pin<Box<CStr>>>,

    features: Vec<Box<CStr>>,
    features_array: Vec<*const c_char>,

    raw_descriptor: clap_plugin_descriptor,
}

// SAFETY: PluginDescriptor is fully self-contained, the pointers refer to data owned by it.
unsafe impl Send for PluginDescriptor {}

// SAFETY: PluginDescriptor does not have any interior mutability.
unsafe impl Sync for PluginDescriptor {}

const EMPTY: &CStr = c"";
const EMPTY_FEATURES: &[*const c_char] = &[core::ptr::null()];

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
    pub fn new(id: &str, name: &str) -> Self {
        if id.is_empty() {
            panic!("Plugin ID must not be blank!");
        }

        if name.is_empty() {
            panic!("Plugin Name must not be blank!");
        }

        let id = Pin::new(
            CString::new(id)
                .expect("Invalid Plugin ID")
                .into_boxed_c_str(),
        );

        let name = Pin::new(
            CString::new(name)
                .expect("Invalid Plugin Name")
                .into_boxed_c_str(),
        );

        Self {
            raw_descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: id.as_ptr(),
                name: name.as_ptr(),
                vendor: EMPTY.as_ptr(),
                url: EMPTY.as_ptr(),
                manual_url: EMPTY.as_ptr(),
                support_url: EMPTY.as_ptr(),
                version: EMPTY.as_ptr(),
                description: EMPTY.as_ptr(),
                features: EMPTY_FEATURES.as_ptr(),
            },

            id,
            name,

            vendor: None,
            url: None,
            manual_url: None,
            support_url: None,
            version: None,
            description: None,

            features: vec![],
            features_array: vec![],
        }
    }

    /// The unique identifier of a plugin. This field is **mandatory**, and should not be blank.
    ///
    /// This identifier should be as globally-unique as possible to any users that might load this
    /// plugin, as this is the key hosts will use to differentiate between different plugins.
    ///
    /// Example: `com.u-he.diva`.
    #[inline]
    pub fn id(&self) -> &CStr {
        &self.id
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

        let id = Pin::new(
            CString::new(id)
                .expect("Invalid Plugin ID")
                .into_boxed_c_str(),
        );

        self.raw_descriptor.id = id.as_ptr();
        self.id = id;

        self
    }

    /// The user-facing display name of this plugin. This field is **mandatory**, and should not be blank.
    ///
    /// This name will be displayed in plugin lists and selectors, and will be the main way users
    /// will find and differentiate the plugin.
    ///
    /// Example: `Diva`.
    #[inline]
    pub fn name(&self) -> &CStr {
        &self.name
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

        let name = Pin::new(
            CString::new(name)
                .expect("Invalid Plugin name")
                .into_boxed_c_str(),
        );

        self.raw_descriptor.name = name.as_ptr();
        self.name = name;

        self
    }

    /// The vendor of the plugin.
    ///
    /// Example: `u-he`.
    #[inline]
    pub fn vendor(&self) -> Option<&CStr> {
        self.vendor.as_deref()
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
        if vendor.is_empty() {
            self.raw_descriptor.vendor = EMPTY.as_ptr();
            self.vendor = None;
        } else {
            let vendor = Pin::new(
                CString::new(vendor)
                    .expect("Invalid Plugin vendor")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.vendor = vendor.as_ptr();
            self.vendor = Some(vendor);
        }

        self
    }

    /// The URL of this plugin's homepage.
    ///
    /// Example: `https://u-he.com/products/diva/`.
    #[inline]
    pub fn url(&self) -> Option<&CStr> {
        self.url.as_deref()
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
        if url.is_empty() {
            self.raw_descriptor.url = EMPTY.as_ptr();
            self.url = None;
        } else {
            let url = Pin::new(
                CString::new(url)
                    .expect("Invalid Plugin URL")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.url = url.as_ptr();
            self.url = Some(url);
        }

        self
    }

    /// The URL of this plugin's user's manual.
    ///
    /// Example: `https://dl.u-he.com/manuals/plugins/diva/Diva-user-guide.pdf`.
    #[inline]
    pub fn manual_url(&self) -> Option<&CStr> {
        self.manual_url.as_deref()
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
        if manual_url.is_empty() {
            self.raw_descriptor.manual_url = EMPTY.as_ptr();
            self.manual_url = None;
        } else {
            let manual_url = Pin::new(
                CString::new(manual_url)
                    .expect("Invalid Plugin Manual URL")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.manual_url = manual_url.as_ptr();
            self.manual_url = Some(manual_url);
        }

        self
    }

    /// The URL of this plugin's support page.
    ///
    /// Example: `https://u-he.com/support/`.
    #[inline]
    pub fn support_url(&self) -> Option<&CStr> {
        self.support_url.as_deref()
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
        if support_url.is_empty() {
            self.raw_descriptor.support_url = EMPTY.as_ptr();
            self.support_url = None;
        } else {
            let support_url = Pin::new(
                CString::new(support_url)
                    .expect("Invalid Plugin Support URL")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.support_url = support_url.as_ptr();
            self.support_url = Some(support_url);
        }

        self
    }

    /// The version string of this plugin.
    ///
    /// While Semver-compatible version strings are preferred, this field can contain any arbitrary
    /// string.
    ///
    /// Example: `1.4.4`.
    #[inline]
    pub fn version(&self) -> Option<&CStr> {
        self.version.as_deref()
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
        if version.is_empty() {
            self.raw_descriptor.version = EMPTY.as_ptr();
            self.version = None;
        } else {
            let version = Pin::new(
                CString::new(version)
                    .expect("Invalid Plugin version")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.version = version.as_ptr();
            self.version = Some(version);
        }

        self
    }

    /// A short description of this plugin.
    ///
    /// Example: `The spirit of analogue`.
    #[inline]
    pub fn description(&self) -> Option<&CStr> {
        self.description.as_deref()
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
        if description.is_empty() {
            self.raw_descriptor.description = EMPTY.as_ptr();
            self.description = None;
        } else {
            let description = Pin::new(
                CString::new(description)
                    .expect("Invalid Plugin description")
                    .into_boxed_c_str(),
            );

            self.raw_descriptor.description = description.as_ptr();
            self.description = Some(description);
        }

        self
    }

    /// An arbitrary list of tags that can be used by hosts to classify this plugin.
    ///
    /// For some standard features, see the constants in the [`features`](super::features) module.
    ///
    /// Example: `"instrument", "synthesizer", "stereo"`.
    #[inline]
    pub fn features(&self) -> &[Box<CStr>] {
        &self.features
    }

    /// Sets the plugin's feature list.
    ///
    /// See the [`features`](PluginDescriptor::features) method documentation for more information.
    pub fn with_features<'a>(mut self, features: impl IntoIterator<Item = &'a CStr>) -> Self {
        self.features = features
            .into_iter()
            .map(|s| CString::from(s).into_boxed_c_str())
            .collect();

        self.features_array = self.features.iter().map(|f| f.as_ptr()).collect();
        self.features_array.push(core::ptr::null());

        self.raw_descriptor.features = self.features_array.as_ptr();

        self
    }

    /// Returns the plugin descriptor as a reference to the C-FFI compatible CLAP struct.
    #[inline]
    pub fn as_raw(&self) -> &clap_plugin_descriptor {
        &self.raw_descriptor
    }
}

impl Clone for PluginDescriptor {
    fn clone(&self) -> Self {
        let id = self.id.clone();
        let name = self.name.clone();

        let vendor = self.vendor.clone();
        let url = self.url.clone();
        let manual_url = self.manual_url.clone();
        let support_url = self.support_url.clone();
        let version = self.version.clone();
        let description = self.description.clone();

        let features = self.features.clone();
        let mut features_array: Vec<_> = features.iter().map(|f| f.as_ptr()).collect();

        if !features_array.is_empty() {
            features_array.push(core::ptr::null())
        }

        Self {
            raw_descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: id.as_ptr(),
                name: name.as_ptr(),

                vendor: vendor.as_deref().unwrap_or(EMPTY).as_ptr(),
                url: url.as_deref().unwrap_or(EMPTY).as_ptr(),
                manual_url: manual_url.as_deref().unwrap_or(EMPTY).as_ptr(),
                support_url: support_url.as_deref().unwrap_or(EMPTY).as_ptr(),
                version: version.as_deref().unwrap_or(EMPTY).as_ptr(),
                description: description.as_deref().unwrap_or(EMPTY).as_ptr(),

                features: if features.is_empty() {
                    EMPTY_FEATURES.as_ptr()
                } else {
                    features_array.as_ptr()
                },
            },

            id,
            name,

            vendor,
            url,
            manual_url,
            support_url,
            version,
            description,

            features,
            features_array,
        }
    }
}
