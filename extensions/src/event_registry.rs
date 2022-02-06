use clack_common::extensions::{Extension, HostExtension};
use clack_host::wrapper::HostWrapper;
use clap_sys::ext::event_registry::{clap_host_event_registry, CLAP_EXT_EVENT_REGISTRY};

#[repr(C)]
pub struct HostEventRegistry {
    inner: clap_host_event_registry,
}

unsafe impl Extension for HostEventRegistry {
    const IDENTIFIER: &'static [u8] = CLAP_EXT_EVENT_REGISTRY;
    type ExtensionType = HostExtension;
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
                unsafe { (self.inner.query?)(host.shared().as_raw(), S::NAME.as_ptr(), &mut out) };

            if !success {
                return None;
            };

            unsafe { Some(EventSpaceId::new(out)?.into_unchecked()) }
        }
    }
};

#[cfg(feature = "clack-host")]
const _: () = {
    use clack_common::events::spaces::{EventSpace, EventSpaceId};
    use clack_common::extensions::ExtensionImplementation;
    use clack_host::host::PluginHoster;
    use clap_sys::host::clap_host;
    use std::ffi::CStr;

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
            unsafe { self.query(S::NAME).map(|i| i.into_unchecked()) }
        }
    }

    impl<'a, H: PluginHoster<'a> + HostEventRegistryImpl> ExtensionImplementation<H>
        for HostEventRegistry
    {
        const IMPLEMENTATION: &'static Self = &HostEventRegistry {
            inner: clap_host_event_registry {
                query: Some(query::<H>),
            },
        };
    }

    unsafe extern "C" fn query<'a, H: PluginHoster<'a> + HostEventRegistryImpl>(
        host: *const clap_host,
        space_name: *const ::std::os::raw::c_char,
        space_id: *mut u16,
    ) -> bool {
        HostWrapper::<H>::handle(host, |host| {
            let space_name = CStr::from_ptr(space_name);

            let result = host.main_thread().as_ref().query(space_name);
            *space_id = EventSpaceId::optional_id(&result);

            Ok(result.is_some())
        })
        .unwrap_or(false)
    }
};
