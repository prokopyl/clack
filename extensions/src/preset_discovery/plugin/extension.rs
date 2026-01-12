use crate::preset_discovery::{HostPresetLoad, PluginPresetLoad, preset_data::Location};
use crate::utils::{cstr_from_nullable_ptr, cstr_to_nullable_ptr};
use clack_plugin::extensions::prelude::*;
use clap_sys::ext::preset_load::*;
use std::ffi::{CStr, c_char};

impl HostPresetLoad {
    /// Report to the host that an error occurred while loading a preset.
    ///
    /// `error_code` is the operating system error, as returned by e.g. [`std::io::Error::raw_os_error`], if applicable.
    /// If not applicable, it should be set to a non-error value, e.g. 0 on Unix and Windows.
    #[inline]
    pub fn on_error(
        &self,
        host: &mut HostMainThreadHandle,
        location: Location,
        load_key: Option<&CStr>,
        os_error: i32,
        message: Option<&CStr>,
    ) {
        if let Some(on_error) = host.use_extension(&self.0).on_error {
            let (kind, path) = location.to_raw();
            // SAFETY: Host pointer comes from HostMainThreadHandle, string pointers come from &CStr, so they are all valid.
            unsafe {
                on_error(
                    host.as_raw(),
                    kind,
                    path,
                    cstr_to_nullable_ptr(load_key),
                    os_error,
                    cstr_to_nullable_ptr(message),
                )
            }
        }
    }

    /// Informs the host that a given preset has been loaded.
    ///
    /// This can be used to e.g. keep the host preset browser in sync with the plugin's.
    #[inline]
    pub fn loaded(
        &self,
        host: &mut HostMainThreadHandle,
        location: Location,
        load_key: Option<&CStr>,
    ) {
        if let Some(loaded) = host.use_extension(&self.0).loaded {
            let (kind, path) = location.to_raw();
            // SAFETY: Host pointer comes from HostMainThreadHandle, string pointers come from &CStr, so they are all valid.
            unsafe { loaded(host.as_raw(), kind, path, cstr_to_nullable_ptr(load_key)) }
        }
    }
}

/// Implementation of the plugin side of the Preset Load extension.
pub trait PluginPresetLoadImpl {
    /// Loads a preset from a given `location`.
    ///
    /// If the given location contains multiple presets, `load_key` can be passed to further
    /// identify the preset to load.
    ///
    /// The `location` and `load_key` should come from a [preset discovery provider](crate::preset_discovery::provider).
    ///
    /// # Errors
    ///
    /// If the preset failed to load for any reason, a [`PluginError`] can be returned.
    fn load_from_location(
        &mut self,
        location: Location,
        load_key: Option<&CStr>,
    ) -> Result<(), PluginError>;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginPresetLoad
where
    P: for<'a> Plugin<MainThread<'a>: PluginPresetLoadImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_preset_load {
            from_location: Some(from_location::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn from_location<P>(
    plugin: *const clap_plugin,
    location_kind: u32,
    location_path: *const c_char,
    load_key: *const c_char,
) -> bool
where
    P: for<'a> Plugin<MainThread<'a>: PluginPresetLoadImpl>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let location = Location::from_raw(location_kind, location_path).ok_or(
            PluginWrapperError::InvalidParameter("Unknown location kind"),
        )?;

        let load_key = cstr_from_nullable_ptr(load_key);

        plugin
            .main_thread()
            .as_mut()
            .load_from_location(location, load_key)?;

        Ok(())
    })
    .is_some()
}
