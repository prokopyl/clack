use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clap_sys::ext::event_registry::{clap_host_event_registry, CLAP_EXT_EVENT_REGISTRY};
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostEventRegistry(RawExtension<HostExtensionSide, clap_host_event_registry>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostEventRegistry {
    const IDENTIFIER: &'static CStr = CLAP_EXT_EVENT_REGISTRY;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-plugin")]
const _: () = {
    use clack_common::events::spaces::{EventSpace, EventSpaceId};
    use clack_plugin::host::HostMainThreadHandle;

    impl HostEventRegistry {
        pub fn query<'a, S: EventSpace<'a>>(
            &self,
            host: &HostMainThreadHandle,
        ) -> Option<EventSpaceId<S>> {
            let mut out = u16::MAX;
            let success =
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { host.use_extension(&self.0).query?(host.as_raw(), S::NAME.as_ptr(), &mut out) };

            if !success {
                return None;
            };

            // SAFETY: the EventSpaceId has been fetched from S's name.
            unsafe { Some(EventSpaceId::new(out)?.into_unchecked()) }
        }
    }
};

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_common::events::spaces::{EventSpace, EventSpaceId};
    use clack_host::extensions::prelude::*;
    use std::os::raw::c_char;

    /// Host implementation of an event registry
    ///
    /// # Safety
    ///
    /// The implementation of the [`query`](HostEventRegistryImpl) method must return stable, unique
    /// event space ids.
    pub unsafe trait HostEventRegistryImpl {
        fn query(&self, space_name: &CStr) -> Option<EventSpaceId>;

        #[inline]
        fn query_type<'a, S: EventSpace<'a>>(&self) -> Option<EventSpaceId<S>> {
            // SAFETY: the EventSpaceId has been fetched from S's name.
            unsafe { self.query(S::NAME).map(|i| i.into_unchecked()) }
        }
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostEventRegistry
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostEventRegistryImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_event_registry {
                query: Some(query::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn query<H: HostHandlers>(
        host: *const clap_host,
        space_name: *const c_char,
        space_id: *mut u16,
    ) -> bool
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostEventRegistryImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            let space_name = CStr::from_ptr(space_name);

            let result = host.main_thread().as_ref().query(space_name);
            *space_id = EventSpaceId::optional_id(&result);

            Ok(result.is_some())
        })
        .unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
