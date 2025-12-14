use clack_common::entry::EntryDescriptor;
use clack_common::factory::{Factory, RawFactoryPointer};
use clack_common::utils::ClapVersion;
use clack_plugin::entry::{Entry, EntryFactories};
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

    pub fn get_factory<'a, F: Factory<'a>>(&'a self) -> Option<F> {
        let mut builder = EntryFactories::new(F::IDENTIFIERS);
        self.inner.declare_factories(&mut builder);

        let found = builder.found()?;

        // SAFETY: The Clack APIs guarantee this pointer is valid and match the given type
        unsafe { Some(F::from_raw(RawFactoryPointer::from_raw(found.cast()))) }
    }
}
