use super::*;
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

/// Implementation of the Plugin-side of the Note Name extension.
pub trait PluginNoteNameImpl {
    /// Returns the number of available [`NoteName`]s.
    fn count(&self) -> usize;

    /// Retrieves a specific [`NoteName`] from its index.
    ///
    /// The plugin gets passed a host-provided mutable buffer to write the configuration into, to
    /// avoid any unnecessary allocations.
    fn get(&self, index: usize, writer: &mut NoteNameWriter);
}

impl<P: Plugin> ExtensionImplementation<P> for PluginNoteName
where
    for<'a> P::MainThread<'a>: PluginNoteNameImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &Self(clap_plugin_note_name {
        count: Some(count::<P>),
        get: Some(get::<P>),
    });
}

unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin) -> u32
where
    for<'a> P::MainThread<'a>: PluginNoteNameImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_ref().count() as u32)).unwrap_or(0)
}

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
        p.main_thread().as_ref().get(index as usize, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

/// An helper struct to write an [`NoteName`] into the host's provided buffer.
pub struct NoteNameWriter<'a> {
    buf: &'a mut MaybeUninit<clap_note_name>,
    is_set: bool,
}

impl<'a> NoteNameWriter<'a> {
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

        unsafe {
            write_to_array_buf(addr_of_mut!((*buf).name), data.name);

            write(addr_of_mut!((*buf).port), data.port);
            write(addr_of_mut!((*buf).channel), data.channel);
            write(addr_of_mut!((*buf).key), data.key);
        }

        self.is_set = true;
    }
}

impl HostNoteName {
    /// Informs the host that the available Note Name list has changed and needs to
    /// be rescanned.
    #[inline]
    pub fn changed(&self, host: &mut HostMainThreadHandle) {
        if let Some(changed) = self.0.changed {
            unsafe { changed(host.as_raw()) }
        }
    }
}
