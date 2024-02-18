use super::*;
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

/// A host-provided buffer for the plugin to write a Note Name in.
#[derive(Clone)]
pub struct NoteNameBuffer {
    inner: MaybeUninit<clap_note_name>,
}

impl Default for NoteNameBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NoteNameBuffer {
    /// Creates an uninitialized Note Name buffer.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl PluginNoteName {
    /// Returns the number of available [`NoteName`]s.
    pub fn count(&self, plugin: &mut PluginMainThreadHandle) -> usize {
        match self.0.count {
            None => 0,
            Some(count) => unsafe { count(plugin.as_raw()) as usize },
        }
    }

    /// Retrieves a specific [`NoteName`] from its index.
    ///
    /// The plugin gets passed a mutable buffer to write the configuration into, to avoid any
    /// unnecessary allocations.
    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: usize,
        buffer: &'b mut NoteNameBuffer,
    ) -> Option<NoteName<'b>> {
        let success =
            unsafe { (self.0.get?)(plugin.as_raw(), index as u32, buffer.inner.as_mut_ptr()) };

        if success {
            Some(unsafe { NoteName::from_raw(buffer.inner.assume_init_ref()) })
        } else {
            None
        }
    }
}

/// Implementation of the Host-side of the Note Name extension.
pub trait HostNoteNameImpl {
    /// Informs the host that the available Note Names list has changed and needs to
    /// be rescanned.
    fn changed(&mut self);
}

impl<H: Host> ExtensionImplementation<H> for HostNoteName
where
    for<'h> <H as Host>::MainThread<'h>: HostNoteNameImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &HostNoteName(clap_host_note_name {
        changed: Some(changed::<H>),
    });
}

unsafe extern "C" fn changed<H: Host>(host: *const clap_host)
where
    for<'a> <H as Host>::MainThread<'a>: HostNoteNameImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread().as_mut().changed();

        Ok(())
    });
}
