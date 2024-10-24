use super::*;
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

/// Implementation of the Plugin-side of the Note Name extension.
pub trait PluginNoteNameImpl {
    /// Returns the number of available [`NoteName`]s.
    fn count(&mut self) -> usize;

    /// Retrieves a specific [`NoteName`] from its index.
    ///
    /// The plugin gets passed a host-provided mutable buffer to write the configuration into, to
    /// avoid any unnecessary allocations.
    fn get(&mut self, index: usize, writer: &mut NoteNameWriter);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginNoteName
where
    for<'a> P::MainThread<'a>: PluginNoteNameImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_note_name {
            count: Some(count::<P>),
            get: Some(get::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin) -> u32
where
    for<'a> P::MainThread<'a>: PluginNoteNameImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_mut().count() as u32)).unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get<P: Plugin>(
    plugin: *const clap_plugin,
    index: u32,
    config: *mut clap_note_name,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginNoteNameImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if config.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_note_name output"));
        };

        let mut writer = NoteNameWriter::from_raw(config);
        p.main_thread().as_mut().get(index as usize, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

/// A helper struct to write an [`NoteName`] into the host's provided buffer.
pub struct NoteNameWriter<'a> {
    buf: &'a mut MaybeUninit<clap_note_name>,
    is_set: bool,
}

impl NoteNameWriter<'_> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is aligned and points to a valid allocation.
    /// However, it doesn't have to be initialized.
    #[inline]
    unsafe fn from_raw(raw: *mut clap_note_name) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    /// Writes the given [`NoteName`] into the host's buffer.
    #[inline]
    pub fn write(&mut self, data: &NoteName) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        // SAFETY: all pointers come from `buf`, which is valid for writes and well-aligned
        unsafe {
            write_to_array_buf(addr_of_mut!((*buf).name), data.name);

            write(addr_of_mut!((*buf).port), data.port.to_raw());
            write(addr_of_mut!((*buf).channel), data.channel.to_raw());
            write(addr_of_mut!((*buf).key), data.key.to_raw());
        }

        self.is_set = true;
    }
}

impl HostNoteName {
    /// Informs the host that the available Note Name list has changed and needs to
    /// be rescanned.
    #[inline]
    pub fn changed(&self, host: &mut HostMainThreadHandle) {
        if let Some(changed) = host.use_extension(&self.0).changed {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { changed(host.as_raw()) }
        }
    }
}
