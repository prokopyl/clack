use super::sys::*;
use super::{PluginAsAuv2Factory, PluginInfoAsAUv2};
use clack_plugin::factory::{FactoryImplementation, FactoryWrapper};
use std::ffi::CStr;

/// A Plugin Factory (CLAP-as-AUv2) implementation.
///
/// See the [module documentation](self) to learn more about the role of this factory.
pub trait PluginFactoryAsAUv2Impl {
    /// Returns the extra AUv2 information for the plugin with the given index, if available.
    fn get_auv2_info(&self, index: u32) -> Option<PluginInfoAsAUv2>;
}

/// A wrapper around a given [`PluginFactoryAsAUv2Impl`] implementation.
///
/// This wrapper is required in order to expose a C FFI-compatible factory to the host.
#[repr(C)]
pub struct PluginFactoryAsAUv2Wrapper<F> {
    inner: FactoryWrapper<clap_plugin_factory_as_auv2, F>,
}

impl<F: PluginFactoryAsAUv2Impl> PluginFactoryAsAUv2Wrapper<F> {
    /// Creates a new [`PluginFactoryAsAUv2Wrapper`] with the given manufacturer info (name and four-character code) and factory implementation.
    ///
    /// # Parameters
    /// * `manufacturer_code`: a four-character code identifying the plugin manufacturer.
    /// * `manufacturer_name`: the name of the plugin manufacturer.
    /// * `factory`: the factory implementation.
    pub const fn new(
        manufacturer_code: &'static CStr,
        manufacturer_name: &'static CStr,
        factory: F,
    ) -> Self {
        Self {
            inner: FactoryWrapper::new(
                clap_plugin_factory_as_auv2 {
                    get_auv2_info: Some(Self::get_auv2_info),
                    manufacturer_code: manufacturer_code.as_ptr(),
                    manufacturer_name: manufacturer_name.as_ptr(),
                },
                factory,
            ),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get_auv2_info(
        factory: *mut clap_plugin_factory_as_auv2,
        index: u32,
        info: *mut clap_plugin_info_as_auv2,
    ) -> bool {
        FactoryWrapper::<_, F>::handle(factory, |factory| {
            if let Some(info_data) = factory.get_auv2_info(index) {
                // SAFETY: the host guarantees that info is well-aligned and valid for writes
                unsafe { info.write(info_data.inner) };
                Ok(true)
            } else {
                Ok(false)
            }
        })
        .unwrap_or(false)
    }
}

// SAFETY: The returned raw implementation matches the spec for clap_plugin_factory
unsafe impl<'a, F: 'a> FactoryImplementation<'a> for PluginFactoryAsAUv2Wrapper<F> {
    type Factory = PluginAsAuv2Factory<'a>;
    type Wrapped = F;

    #[inline]
    fn wrapper(&self) -> &FactoryWrapper<clap_plugin_factory_as_auv2, F> {
        &self.inner
    }
}
