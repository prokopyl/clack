use crate::extensions::wrapper::handle_panic;
use crate::factory::error::FactoryWrapperError;
use std::panic::AssertUnwindSafe;

#[repr(C)]
pub struct FactoryWrapper<Raw, F> {
    raw: Raw,
    inner: F,
}

impl<Raw, F> FactoryWrapper<Raw, F> {
    #[inline]
    pub const fn new(raw: Raw, inner: F) -> Self {
        Self { raw, inner }
    }

    #[inline]
    pub const fn as_raw(&self) -> &Raw {
        &self.raw
    }

    #[inline]
    pub const fn factory(&self) -> &F {
        &self.inner
    }

    /// # Safety
    /// The plugin factory pointer must be valid
    pub unsafe fn handle<T>(
        raw: *const Raw,
        handler: impl FnOnce(&F) -> Result<T, FactoryWrapperError>,
    ) -> Option<T> {
        let factory = Self::from_raw(raw);
        let result = factory.and_then(|factory| {
            match handle_panic(AssertUnwindSafe(|| handler(factory.factory()))) {
                Err(_) => Err(FactoryWrapperError::Panic),
                Ok(Err(e)) => Err(e),
                Ok(Ok(val)) => Ok(val),
            }
        });

        match result {
            Ok(value) => Some(value),
            Err(e) => {
                eprintln!("[CLAP_PLUGIN_FACTORY_ERROR] {e}");

                None
            }
        }
    }

    /// # Safety
    /// The plugin factory pointer must be valid (but it can be null)
    unsafe fn from_raw<'a>(raw: *const Raw) -> Result<&'a Self, FactoryWrapperError> {
        (raw as *const Self)
            .as_ref()
            .ok_or(FactoryWrapperError::NullFactoryInstance)
    }
}
