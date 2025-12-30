use crate::preset_discovery::{HostPresetLoad, Location, PluginPresetLoad};
use crate::utils::{cstr_from_nullable_ptr, cstr_to_nullable_ptr};
use clack_plugin::extensions::prelude::*;
use clap_sys::ext::preset_load::*;
use std::ffi::{CStr, c_char};

impl HostPresetLoad {
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
            // SAFETY: TODO
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

    #[inline]
    pub fn loaded(
        &self,
        host: &mut HostMainThreadHandle,
        location: Location,
        load_key: Option<&CStr>,
    ) {
        if let Some(loaded) = host.use_extension(&self.0).loaded {
            let (kind, path) = location.to_raw();
            // SAFETY: TODO
            unsafe { loaded(host.as_raw(), kind, path, cstr_to_nullable_ptr(load_key)) }
        }
    }
}

pub trait PluginPresetLoadImpl {
    fn load_from_location(
        &mut self,
        location: Location,
        load_key: Option<&CStr>,
    ) -> Result<(), PluginError>;
}

// SAFETY: TODO
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
