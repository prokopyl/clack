use super::sys::*;
use super::{PluginAsVST3, PluginFactoryAsVST3, PluginInfoAsVST3, SupportedNoteExpressions};
use clack_plugin::extensions::prelude::*;
use clack_plugin::factory::{FactoryImplementation, FactoryWrapper};
use std::ffi::CStr;

pub trait PluginFactoryAsVST3Impl {
    fn get_vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3<'_>>;
}

#[repr(C)]
pub struct PluginFactoryAsVST3Wrapper<F> {
    inner: FactoryWrapper<clap_plugin_factory_as_vst3, F>,
}

impl<F: PluginFactoryAsVST3Impl> PluginFactoryAsVST3Wrapper<F> {
    #[inline]
    pub const fn new(
        vendor: Option<&'static CStr>,
        vendor_url: Option<&'static CStr>,
        email_contact: Option<&'static CStr>,
        factory: F,
    ) -> Self {
        Self {
            inner: FactoryWrapper::new(
                clap_plugin_factory_as_vst3 {
                    vendor: match vendor {
                        Some(v) => v.as_ptr(),
                        None => core::ptr::null(),
                    },
                    vendor_url: match vendor_url {
                        Some(v) => v.as_ptr(),
                        None => core::ptr::null(),
                    },
                    email_contact: match email_contact {
                        Some(v) => v.as_ptr(),
                        None => core::ptr::null(),
                    },
                    get_vst3_info: Some(Self::get_vst3_info),
                },
                factory,
            ),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get_vst3_info(
        factory: *mut clap_plugin_factory_as_vst3,
        index: u32,
    ) -> *const clap_plugin_info_as_vst3 {
        FactoryWrapper::<_, F>::handle(factory, |factory| {
            Ok(factory.get_vst3_info(index).map(|i| i.as_raw() as *const _))
        })
        .flatten()
        .unwrap_or(core::ptr::null())
    }
}

pub trait PluginAsVST3Impl {
    fn num_midi_channels(&mut self, note_port: u32) -> u32;
    fn supported_note_expressions(&mut self) -> SupportedNoteExpressions;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginAsVST3
where
    for<'a> P: Plugin<MainThread<'a>: PluginAsVST3Impl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_as_vst3 {
            get_num_midi_channels: Some(get_num_midi_channels::<P>),
            supported_note_expressions: Some(supported_note_expressions::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_num_midi_channels<P>(plugin: *const clap_plugin, note_port: u32) -> u32
where
    for<'a> P: Plugin<MainThread<'a>: PluginAsVST3Impl>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().num_midi_channels(note_port))
    })
    .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn supported_note_expressions<P>(plugin: *const clap_plugin) -> u32
where
    for<'a> P: Plugin<MainThread<'a>: PluginAsVST3Impl>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin
            .main_thread()
            .as_mut()
            .supported_note_expressions()
            .bits())
    })
    .unwrap_or(0)
}

// SAFETY: The returned raw implementation matches the spec for clap_plugin_factory_as_vst3
unsafe impl<'a, F: 'a> FactoryImplementation<'a> for PluginFactoryAsVST3Wrapper<F> {
    type Factory = PluginFactoryAsVST3<'a>;
    type Wrapped = F;

    #[inline]
    fn wrapper(&self) -> &FactoryWrapper<clap_plugin_factory_as_vst3, Self::Wrapped> {
        &self.inner
    }
}
