use clack_common::extensions::{Extension, HostExtension};
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
