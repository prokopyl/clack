use crate::preset_discovery::prelude::*;
use crate::utils::{cstr_from_nullable_ptr, cstr_to_nullable_ptr};
use clack_host::extensions::prelude::*;
use clap_sys::ext::preset_load::*;
use clap_sys::factory::preset_discovery::clap_preset_discovery_location_kind;
use std::error::Error;
use std::ffi::{CStr, c_char};
use std::fmt::{Display, Formatter};

impl PluginPresetLoad {
    /// Loads a preset from a given `location`.
    ///
    /// If the given location contains multiple presets, `load_key` can be passed to further
    /// identify the preset to load.
    ///
    /// The `location` and `load_key` can be discovered from a plugin's [preset finder provider](crate::preset_discovery::provider).
    ///
    /// # Errors
    ///
    /// If the preset failed to load for any reason, a [`PresetLoadError`] is returned.
    ///
    /// That type does not contain any error information, however the plugin may have called
    /// [`HostPresetLoadImpl::on_error`] with more information about the error.
    #[inline]
    pub fn load_from_location(
        &self,
        plugin: &mut PluginMainThreadHandle,
        location: Location,
        load_key: Option<&CStr>,
    ) -> Result<(), PresetLoadError> {
        let Some(from_location) = plugin.use_extension(&self.0).from_location else {
            return Err(PresetLoadError { _inner: () });
        };

        let (kind, path) = location.to_raw();
        let load_key = cstr_to_nullable_ptr(load_key);

        // SAFETY: kind, path and load_key are valid as they come from &CStr references
        // the plugin pointer is valid as enforced by PluginMainThreadHandle
        let success = unsafe { from_location(plugin.as_raw_ptr(), kind, path, load_key) };

        if success {
            Ok(())
        } else {
            Err(PresetLoadError { _inner: () })
        }
    }
}

/// Error that can occur when loading a preset.
///
/// See [`PluginPresetLoad::load_from_location`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PresetLoadError {
    _inner: (),
}

impl Display for PresetLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to load preset from location")
    }
}

impl Error for PresetLoadError {}

/// Implementation of the host side of the Preset Load extension.
pub trait HostPresetLoadImpl {
    /// Report to the host that an error occurred while loading a preset.
    ///
    /// `error_code` is the operating system error, as returned by e.g. [`std::io::Error::raw_os_error`], if applicable.
    /// If not applicable, it should be set to a non-error value, e.g. 0 on Unix and Windows.
    fn on_error(
        &mut self,
        location: Location,
        load_key: Option<&CStr>,
        os_error: i32,
        message: Option<&CStr>,
    );

    /// Informs the host that a given preset has been loaded.
    ///
    /// This can be used to e.g. keep the host preset browser in sync with the plugin's.
    fn loaded(&mut self, location: Location, load_key: Option<&CStr>);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostPresetLoad
where
    for<'a> H: HostHandlers<MainThread<'a>: HostPresetLoadImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_preset_load {
            loaded: Some(loaded::<H>),
            on_error: Some(on_error::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn loaded<H>(
    host: *const clap_host,
    kind: clap_preset_discovery_location_kind,
    path: *const c_char,
    load_key: *const c_char,
) where
    for<'a> H: HostHandlers<MainThread<'a>: HostPresetLoadImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        // SAFETY: path is guaranteed to be either NULL or valid by the CLAP spec.
        let location = unsafe { Location::from_raw(kind, path) }
            .ok_or(HostWrapperError::InvalidParameter("Invalid location"))?;

        // SAFETY: load_key is guaranteed to be either NULL or valid by the CLAP spec.
        let load_key = unsafe { cstr_from_nullable_ptr(load_key) };

        host.main_thread().as_mut().loaded(location, load_key);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn on_error<H>(
    host: *const clap_host,
    kind: clap_preset_discovery_location_kind,
    path: *const c_char,
    load_key: *const c_char,
    os_error: i32,
    message: *const c_char,
) where
    for<'a> H: HostHandlers<MainThread<'a>: HostPresetLoadImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        // SAFETY: path is guaranteed to be either NULL or valid by the CLAP spec.
        let location = unsafe { Location::from_raw(kind, path) }
            .ok_or(HostWrapperError::InvalidParameter("Invalid location"))?;

        // SAFETY: load_key is guaranteed to be either NULL or valid by the CLAP spec.
        let load_key = unsafe { cstr_from_nullable_ptr(load_key) };
        // SAFETY: message is guaranteed to be either NULL or valid by the CLAP spec.
        let message = unsafe { cstr_from_nullable_ptr(message) };

        host.main_thread()
            .as_mut()
            .on_error(location, load_key, os_error, message);

        Ok(())
    });
}
