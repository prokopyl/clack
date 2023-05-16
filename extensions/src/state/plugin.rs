use super::*;
use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::extensions::prelude::*;
use clap_sys::stream::{clap_istream, clap_ostream};

impl HostState {
    #[inline]
    pub fn mark_dirty(&mut self, host: &HostMainThreadHandle) {
        if let Some(mark_dirty) = self.0.mark_dirty {
            unsafe { mark_dirty(host.shared().as_raw()) }
        }
    }
}

pub trait PluginStateImpl {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError>;
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError>;
}

impl<P: Plugin> ExtensionImplementation<P> for PluginState
where
    for<'a> P::MainThread<'a>: PluginStateImpl,
{
    const IMPLEMENTATION: &'static Self = &PluginState(
        clap_plugin_state {
            save: Some(save::<P>),
            load: Some(load::<P>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn load<P: Plugin>(
    plugin: *const clap_plugin,
    stream: *const clap_istream,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginStateImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let input = InputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().load(input)?;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn save<P: Plugin>(
    plugin: *const clap_plugin,
    stream: *const clap_ostream,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginStateImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let output = OutputStream::from_raw_mut(&mut *(stream as *mut _));
        p.main_thread().as_mut().save(output)?;
        Ok(())
    })
    .is_some()
}
