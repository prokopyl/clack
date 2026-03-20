use super::*;
use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::extensions::prelude::*;
use clap_sys::stream::{clap_istream, clap_ostream};

impl HostState {
    /// Tells the host that the plugin state has changed, and may need to be saved again.
    ///
    /// Note that if a parameter value changes, it is implicit that the state is dirty.
    #[inline]
    pub fn mark_dirty(&mut self, host: &HostMainThreadHandle) {
        if let Some(mark_dirty) = host.use_extension(&self.0).mark_dirty {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { mark_dirty(host.as_raw()) }
        }
    }
}

/// Implementation of the Plugin side of the State extension.
pub trait PluginStateImpl {
    /// Saves the plugin state into a given `output` byte stream.
    ///
    /// # Errors
    ///
    /// If this operation fails, any [`PluginError`] can be returned.
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError>;
    /// Loads the plugin state from a given `input` byte stream.
    ///
    /// # Errors
    ///
    /// If this operation fails, any [`PluginError`] can be returned.
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError>;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginState
where
    for<'a> P: Plugin<MainThread<'a>: PluginStateImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_state {
            save: Some(save::<P>),
            load: Some(load::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn load<P>(plugin: *const clap_plugin, stream: *const clap_istream) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginStateImpl>,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let input = InputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().load(input)?;
        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn save<P>(plugin: *const clap_plugin, stream: *const clap_ostream) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginStateImpl>,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let output = OutputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().save(output)?;
        Ok(())
    })
    .is_some()
}
