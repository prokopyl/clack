use crate::factory::FactoryPointer;
use clack_common::entry::EntryDescriptor;
use clack_common::utils::ClapVersion;
use clack_plugin::entry::{Entry, EntryFactories};
use std::ptr::NonNull;
use std::sync::Arc;

trait DynEntry: Send + Sync + 'static {
    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>);
}

impl<E: Entry> DynEntry for E {
    #[inline]
    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
        <E as Entry>::declare_factories(self, builder);
    }
}

#[derive(Clone)]
pub(crate) struct ClackEntry {
    inner: Arc<dyn DynEntry>,
}

impl ClackEntry {
    pub const DUMMY_DESCRIPTOR: EntryDescriptor = EntryDescriptor {
        clap_version: ClapVersion::CURRENT.to_raw(),
        init: None,
        deinit: None,
        get_factory: None,
    };

    #[inline]
    pub fn new<E: Entry>(entry: E) -> Self {
        Self {
            inner: Arc::new(entry),
        }
    }

    pub fn get_factory<'a, F: FactoryPointer<'a>>(&'a self) -> Option<F> {
        let mut builder = EntryFactories::new(F::IDENTIFIER);
        self.inner.declare_factories(&mut builder);
        // SAFETY: The EntryFactories type ensures we have a pointer that matches the given
        // identifier, which comes from `F`. It also ensures the pointer is valid.
        Some(unsafe { F::from_raw(NonNull::new(builder.found().cast_mut())?) })
    }
}
