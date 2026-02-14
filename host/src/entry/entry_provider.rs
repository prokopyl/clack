use clack_common::entry::EntryDescriptor;
use std::ptr::NonNull;
use std::sync::Arc;

/// A type which can provide an entry descriptor pointer for Clack hosts to load.
///
/// # Safety
///
/// Implementors *must* ensure the [`EntryDescriptor`] returned by the [`entry_pointer`](Self::entry_pointer) method
/// remains valid for the entire lifetime of this type. That means all the function pointers it contains
/// must remain valid to call.
///
/// Moreover, the returned [`EntryDescriptor`] must have a stable address. It must not move for the
/// lifetime of this type.
pub unsafe trait EntryProvider: Send + Sync + 'static {
    /// Returns a pointer to the [`EntryDescriptor`] this provides.
    ///
    /// This pointer always valid for reads, as long as this type is alive.
    /// This pointer is also stable, i.e. its address will never change for the entire duration of
    /// this type's lifetime.
    fn entry_pointer(&self) -> NonNull<EntryDescriptor>;
}

// SAFETY: The pointer is always valid since it comes from a 'static reference
unsafe impl EntryProvider for &'static EntryDescriptor {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        (*self).into()
    }
}

// SAFETY: The pointer comes from this Arc, it is always valid as long as the Arc is alive
unsafe impl EntryProvider for Arc<EntryDescriptor> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.as_ref().into()
    }
}

// SAFETY: The pointer comes from this Box, it is always valid as long as the Box is alive
unsafe impl EntryProvider for Box<EntryDescriptor> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.as_ref().into()
    }
}
