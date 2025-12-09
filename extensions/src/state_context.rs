use clack_common::extensions::*;
use clap_sys::ext::state_context::*;
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginStateContext(RawExtension<PluginExtensionSide, clap_plugin_state_context>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginStateContext {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_STATE_CONTEXT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_common::stream::{InputStream, OutputStream};
    use clack_plugin::extensions::prelude::*;
    use clap_sys::ext::log::CLAP_LOG_HOST_MISBEHAVING;
    use clap_sys::stream::{clap_istream, clap_ostream};

    pub trait PluginStateContextImpl {
        fn save(
            &mut self,
            output: &mut OutputStream,
            context_type: StateContextType,
        ) -> Result<(), PluginError>;

        fn load(
            &mut self,
            input: &mut InputStream,
            context_type: StateContextType,
        ) -> Result<(), PluginError>;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P> ExtensionImplementation<P> for PluginStateContext
    where
        P: for<'a> Plugin<MainThread<'a>: PluginStateContextImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_state_context {
                load: Some(load::<P>),
                save: Some(save::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn load<P>(
        plugin: *const clap_plugin,
        stream: *const clap_istream,
        context: clap_plugin_state_context_type,
    ) -> bool
    where
        for<'a> P: Plugin<MainThread<'a>: PluginStateContextImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |p| {
            let input = InputStream::from_raw_mut(&mut *(stream as *mut _));
            let Some(context) = StateContextType::from_raw(context) else {
                return Err(PluginWrapperError::Message(
                    CLAP_LOG_HOST_MISBEHAVING,
                    "Invalid context type",
                ));
            };

            p.main_thread().as_mut().load(input, context)?;
            Ok(())
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn save<P>(
        plugin: *const clap_plugin,
        stream: *const clap_ostream,
        context: clap_plugin_state_context_type,
    ) -> bool
    where
        for<'a> P: Plugin<MainThread<'a>: PluginStateContextImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |p| {
            let Some(context) = StateContextType::from_raw(context) else {
                return Err(PluginWrapperError::Message(
                    CLAP_LOG_HOST_MISBEHAVING,
                    "Invalid context type",
                ));
            };

            let output = OutputStream::from_raw_mut(&mut *(stream as *mut _));
            p.main_thread().as_mut().save(output, context)?;
            Ok(())
        })
        .is_some()
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateContextType {
    ForPreset = CLAP_STATE_CONTEXT_FOR_PRESET,
    ForDuplicate = CLAP_STATE_CONTEXT_FOR_DUPLICATE,
    ForProject = CLAP_STATE_CONTEXT_FOR_PROJECT,
}

impl StateContextType {
    #[inline]
    pub fn from_raw(raw: clap_plugin_state_context_type) -> Option<Self> {
        match raw {
            CLAP_STATE_CONTEXT_FOR_PRESET => Some(Self::ForPreset),
            CLAP_STATE_CONTEXT_FOR_DUPLICATE => Some(Self::ForDuplicate),
            CLAP_STATE_CONTEXT_FOR_PROJECT => Some(Self::ForProject),
            _ => None,
        }
    }

    #[inline]
    pub fn to_raw(self) -> clap_plugin_state_context_type {
        self as u32
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use crate::state::StateError;
    use clack_common::stream::{InputStream, OutputStream};
    use clack_host::extensions::prelude::*;
    use std::io::{Read, Write};

    impl PluginStateContext {
        pub fn load(
            &self,
            plugin: &mut PluginMainThreadHandle,
            reader: &mut impl Read,
            context_type: StateContextType,
        ) -> Result<(), StateError> {
            let mut stream = InputStream::from_reader(reader);

            let load = plugin
                .use_extension(&self.0)
                .load
                .ok_or(StateError::loading())?;

            let success =
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { load(plugin.as_raw(), stream.as_raw_mut(), context_type.to_raw()) };

            // SAFETY: This type ensures the function pointer is valid.
            if success {
                Ok(())
            } else {
                Err(StateError::saving())
            }
        }

        pub fn save(
            &self,
            plugin: &mut PluginMainThreadHandle,
            writer: &mut impl Write,
            context_type: StateContextType,
        ) -> Result<(), StateError> {
            let mut stream = OutputStream::from_writer(writer);

            let save = plugin
                .use_extension(&self.0)
                .save
                .ok_or(StateError::loading())?;

            let success =
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { save(plugin.as_raw(), stream.as_raw_mut(), context_type.to_raw()) };

            if success {
                Ok(())
            } else {
                Err(StateError::saving())
            }
        }
    }
}
