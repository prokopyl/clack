use clack_common::extensions::*;
use clap_sys::ext::param_indication::*;
use std::{ffi::CStr, fmt::Display};

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
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            i if i == CLAP_PARAM_INDICATION_AUTOMATION_NONE => Some(Self::None),
            i if i == CLAP_PARAM_INDICATION_AUTOMATION_PRESENT => Some(Self::Present),
            i if i == CLAP_PARAM_INDICATION_AUTOMATION_PLAYING => Some(Self::Playing),
            i if i == CLAP_PARAM_INDICATION_AUTOMATION_RECORDING => Some(Self::Recording),
            i if i == CLAP_PARAM_INDICATION_AUTOMATION_OVERRIDING => Some(Self::Overriding),
            _ => None,
        }
    }

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

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginParamIndication(RawExtension<PluginExtensionSide, clap_plugin_param_indication>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginParamIndication {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAM_INDICATION;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use std::ffi::CString;

    use super::*;
    use clack_host::extensions::prelude::*;
    use clap_sys::color::clap_color;

    impl PluginParamIndication {
        #[inline]
        pub fn set_mapping(
            &self,
            plugin: &mut PluginMainThreadHandle,
            param_id: ClapId,
            has_mapping: bool,
            color: clap_color,
            label: &str,
            description: &str,
        ) {
            if let Some(set_mapping) = plugin.use_extension(&self.0).set_mapping {
                let label = CString::new(label).unwrap();
                let description = CString::new(description).unwrap();
                // SAFETY: This type ensures the function pointer is valid.
                unsafe {
                    set_mapping(
                        plugin.as_raw(),
                        param_id.get(),
                        has_mapping,
                        &color,
                        label.as_ptr(),
                        description.as_ptr(),
                    )
                }
            }
        }

        #[inline]
        pub fn set_automation(
            &self,
            plugin: &mut PluginMainThreadHandle,
            param_id: ClapId,
            automation_state: ParamIndicationAutomation,
            color: clap_color,
        ) {
            if let Some(set_automation) = plugin.use_extension(&self.0).set_automation {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe {
                    set_automation(
                        plugin.as_raw(),
                        param_id.get(),
                        automation_state.to_raw(),
                        &color,
                    )
                }
            }
        }
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;
    use clap_sys::color::clap_color;

    pub trait PluginParamIndicationImpl {
        fn set_mapping(
            &mut self,
            param_id: ClapId,
            has_mapping: bool,
            color: clap_color,
            label: &CStr,
            description: &CStr,
        );
        fn set_automation(
            &mut self,
            param_id: ClapId,
            automation_state: ParamIndicationAutomation,
            color: clap_color,
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
        let param_id = ClapId::new(param_id);
        let color = *color;
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin.main_thread().as_mut().set_mapping(
                param_id,
                has_mapping,
                color,
                CStr::from_ptr(label),
                CStr::from_ptr(description),
            );

            Ok(())
        });
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
        let param_id = ClapId::new(param_id);
        let automation_state = ParamIndicationAutomation::from_raw(automation_state).unwrap();
        let color = *color;
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin
                .main_thread()
                .as_mut()
                .set_automation(param_id, automation_state, color);
            Ok(())
        });
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
