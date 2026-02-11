use clack_common::entry::EntryDescriptor;
use std::ptr::NonNull;
use std::sync::Arc;

/// A type which can provide an entry descriptor for Clack hosts to load.
///
/// # Safety
///
/// Implementors *must* ensure the [`EntryDescriptor`] returned by the [`entry`](Self::entry) method
/// remains valid for the entire lifetime of this type. That means all the function pointers it contains
/// must remain valid to call.
pub unsafe trait EntryProvider: Send + Sync + 'static {
    fn entry_pointer(&self) -> NonNull<EntryDescriptor>;
}

unsafe impl EntryProvider for &'static EntryDescriptor {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        (*self).into()
    }
}

unsafe impl EntryProvider for Arc<EntryDescriptor> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.as_ref().into()
    }
}

unsafe impl EntryProvider for Box<EntryDescriptor> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.as_ref().into()
    }
}
