use super::*;
use clack_common::extensions::ExtensionImplementation;
use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::host::HostMainThreadHandle;
use clack_plugin::plugin::wrapper::PluginWrapper;
use clack_plugin::plugin::Plugin;
use clack_plugin::prelude::PluginError;
use clap_sys::plugin::clap_plugin;
use clap_sys::stream::{clap_istream, clap_ostream};

impl HostState {
    #[inline]
    pub fn mark_dirty(&mut self, host: &HostMainThreadHandle) {
        if let Some(mark_dirty) = self.0.mark_dirty {
            unsafe { mark_dirty(host.shared().as_raw()) }
        }
    }
}

pub trait PluginStateImplementation {
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError>;
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError>;
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginState
where
    P::MainThread: PluginStateImplementation,
{
    const IMPLEMENTATION: &'static Self = &PluginState(
        clap_plugin_state {
            save: Some(save::<P>),
            load: Some(load::<P>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn load<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    stream: *const clap_istream,
) -> bool
where
    P::MainThread: PluginStateImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let input = InputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().load(input)?;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn save<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    stream: *const clap_ostream,
) -> bool
where
    P::MainThread: PluginStateImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let output = OutputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().save(output)?;
        Ok(())
    })
    .is_some()
}
