//! Allows the host to indicate parameter automation status or controller mapping in the Plugin's
//! own GUI interface.
//!
//! This can be used to indicate:
//! - a physical controller is mapped to a parameter
//! - the parameter is current playing an automation
//! - the parameter is overriding the automation
//! - etc...
//!
//! The color semantic depends upon the host here and the goal is to have a consistent experience
//! across all plugins.

use clack_common::extensions::*;
use clap_sys::ext::param_indication::*;
use std::{ffi::CStr, fmt::Display};

/// Types of automation indication that can be applied to a parameter.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum ParamIndicationAutomation {
    /// Host does not have automation for this parameter
    None = CLAP_PARAM_INDICATION_AUTOMATION_NONE,
    /// Host has automation for this parameter, but is not playing it
    Present = CLAP_PARAM_INDICATION_AUTOMATION_PRESENT,
    /// Host is playing automation for this parameter
    Playing = CLAP_PARAM_INDICATION_AUTOMATION_PLAYING,
    /// Host is recording automation for this parameter
    Recording = CLAP_PARAM_INDICATION_AUTOMATION_RECORDING,
    /// Host should play automation for this parameter, but the user has started to adjust this
    /// parameter and is overriding the automation playback
    Overriding = CLAP_PARAM_INDICATION_AUTOMATION_OVERRIDING,
}

impl ParamIndicationAutomation {
    /// Returns the [`ParamIndicationAutomation`] from its raw, C-FFI compatible representation.
    ///
    /// If the given value doesn't match a known [`ParamIndicationAutomation`], this returns `None` instead.
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            CLAP_PARAM_INDICATION_AUTOMATION_NONE => Some(Self::None),
            CLAP_PARAM_INDICATION_AUTOMATION_PRESENT => Some(Self::Present),
            CLAP_PARAM_INDICATION_AUTOMATION_PLAYING => Some(Self::Playing),
            CLAP_PARAM_INDICATION_AUTOMATION_RECORDING => Some(Self::Recording),
            CLAP_PARAM_INDICATION_AUTOMATION_OVERRIDING => Some(Self::Overriding),
            _ => None,
        }
    }

    /// Returns this [`ParamIndicationAutomation`] as its raw, C-FFI compatible representation.
    #[inline]
    pub fn to_raw(self) -> u32 {
        self as _
    }
}

impl Display for ParamIndicationAutomation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            Self::None => "NONE",
            Self::Present => "PRESENT",
            Self::Playing => "PLAYING",
            Self::Recording => "RECORDING",
            Self::Overriding => "OVERRIDING",
        };

        f.write_str(display_str)
    }
}

/// The Plugin-side of the Param Indication extension.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginParamIndication(RawExtension<PluginExtensionSide, clap_plugin_param_indication>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginParamIndication {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_PARAM_INDICATION, CLAP_EXT_PARAM_INDICATION_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use crate::utils::cstr_to_nullable_ptr;
    use clack_common::utils::Color;
    use clack_host::extensions::prelude::*;

    impl PluginParamIndication {
        /// Sets (or clears) a parameter mapping indication for the given `param_id`.
        ///
        /// # Parameters
        ///
        /// * `has_mapping`: whether the parameter currently has a mapping.
        /// * `color`: if set, the color to use to highlight the control in the plugin GUI.
        /// * `label`: if set, a small string to display on top of the knob, which identifies the hardware controller
        /// * `description`: if set, a longer string which can be used as e.g. a tooltip, which describes the current mapping.
        ///
        /// Note that parameter indications should not be saved in the plugin context, and are off by default.
        #[inline]
        pub fn set_mapping(
            &self,
            plugin: &mut PluginMainThreadHandle,
            param_id: ClapId,
            has_mapping: bool,
            color: Option<Color>,
            label: Option<&CStr>,
            description: Option<&CStr>,
        ) {
            if let Some(set_mapping) = plugin.use_extension(&self.0).set_mapping {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe {
                    set_mapping(
                        plugin.as_raw(),
                        param_id.get(),
                        has_mapping,
                        if let Some(color) = &color {
                            color
                        } else {
                            core::ptr::null()
                        },
                        cstr_to_nullable_ptr(label),
                        cstr_to_nullable_ptr(description),
                    )
                }
            }
        }

        /// Sets (or clears) the `automation_state` associated to a given `param_id`.
        ///
        /// The host can also optionally pass a specific [`Color`] to for the plugin GUI to use for
        /// its automation indication, to keep it consistent with the host's own color scheme.
        ///
        /// Note that parameter indications should not be saved in the plugin context, and are off by default.
        #[inline]
        pub fn set_automation(
            &self,
            plugin: &mut PluginMainThreadHandle,
            param_id: ClapId,
            automation_state: ParamIndicationAutomation,
            color: Option<Color>,
        ) {
            if let Some(set_automation) = plugin.use_extension(&self.0).set_automation {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe {
                    set_automation(
                        plugin.as_raw(),
                        param_id.get(),
                        automation_state.to_raw(),
                        if let Some(color) = &color {
                            color
                        } else {
                            core::ptr::null()
                        },
                    )
                }
            }
        }
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use crate::utils::cstr_from_nullable_ptr;
    use clack_common::utils::Color;
    use clack_plugin::extensions::prelude::*;
    use clap_sys::color::clap_color;

    /// Implementation of the Plugin-side of the Param Indication extension.
    pub trait PluginParamIndicationImpl {
        /// Sets (or clears) a parameter mapping indication for the given `param_id`.
        ///
        /// # Parameters
        ///
        /// * `has_mapping`: whether the parameter currently has a mapping.
        /// * `color`: if set, the color to use to highlight the control in the plugin GUI.
        /// * `label`: if set, a small string to display on top of the knob, which identifies the hardware controller
        /// * `description`: if set, a longer string which can be used as e.g. a tooltip, which describes the current mapping.
        ///
        /// Note that parameter indications should not be saved in the plugin context, and are off by default.
        fn set_mapping(
            &mut self,
            param_id: ClapId,
            has_mapping: bool,
            color: Option<Color>,
            label: Option<&CStr>,
            description: Option<&CStr>,
        );

        /// Sets (or clears) the `automation_state` associated to a given `param_id`.
        ///
        /// The host can also optionally pass a specific [`Color`] to for the plugin GUI to use for
        /// its automation indication, to keep it consistent with the host's own color scheme.
        ///
        /// Note that parameter indications should not be saved in the plugin context, and are off by default.
        fn set_automation(
            &mut self,
            param_id: ClapId,
            automation_state: ParamIndicationAutomation,
            color: Option<Color>,
        );
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginParamIndication
    where
        for<'a> P::MainThread<'a>: PluginParamIndicationImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_param_indication {
                set_mapping: Some(set_mapping::<P>),
                set_automation: Some(set_automation::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn set_mapping<P: Plugin>(
        plugin: *const clap_plugin,
        param_id: u32,
        has_mapping: bool,
        color: *const clap_color,
        label: *const std::ffi::c_char,
        description: *const std::ffi::c_char,
    ) where
        for<'a> P::MainThread<'a>: PluginParamIndicationImpl,
    {
        // SAFETY: panics are caught by PluginWrapper so they don't cross FFI boundary
        unsafe {
            PluginWrapper::<P>::handle(plugin, |plugin| {
                let param_id = ClapId::from_raw(param_id)
                    .ok_or(PluginWrapperError::InvalidParameter("param_id"))?;

                let color = if color.is_null() {
                    None
                } else {
                    Some(color.read())
                };

                plugin.main_thread().as_mut().set_mapping(
                    param_id,
                    has_mapping,
                    color,
                    cstr_from_nullable_ptr(label),
                    cstr_from_nullable_ptr(description),
                );

                Ok(())
            });
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn set_automation<P: Plugin>(
        plugin: *const clap_plugin,
        param_id: u32,
        automation_state: u32,
        color: *const clap_color,
    ) where
        for<'a> P::MainThread<'a>: PluginParamIndicationImpl,
    {
        // SAFETY: panics are caught by PluginWrapper so they don't cross FFI boundary
        unsafe {
            PluginWrapper::<P>::handle(plugin, |plugin| {
                let automation_state = ParamIndicationAutomation::from_raw(automation_state)
                    .ok_or(PluginWrapperError::InvalidParameter("automation_state"))?;
                let param_id = ClapId::from_raw(param_id)
                    .ok_or(PluginWrapperError::InvalidParameter("param_id"))?;

                let color = if color.is_null() {
                    None
                } else {
                    Some(color.read())
                };

                plugin
                    .main_thread()
                    .as_mut()
                    .set_automation(param_id, automation_state, color);
                Ok(())
            });
        }
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
