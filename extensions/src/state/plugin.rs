use super::*;
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::host::HostMainThreadHandle;
use clack_plugin::plugin::wrapper::PluginWrapper;
use clack_plugin::plugin::Plugin;
use clap_sys::plugin::clap_plugin;
use clap_sys::stream::{clap_istream, clap_ostream};

impl HostState {
    #[inline]
    pub fn mark_dirty(&mut self, host: &HostMainThreadHandle) {
        unsafe { (self.0.mark_dirty)(host.shared().as_raw()) }
    }
}

pub trait PluginStateImplementation {
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError>;
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError>;
}

unsafe impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for super::PluginState
where
    P::MainThread: PluginStateImplementation,
{
    type Interface = clap_plugin_state;
    const INTERFACE: &'static Self::Interface = &clap_plugin_state {
        save: save::<P>,
        load: load::<P>,
    };
}

unsafe extern "C" fn load<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    stream: *mut clap_istream,
) -> bool
where
    P::MainThread: PluginStateImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let input = InputStream::from_raw_mut(&mut *stream);
        p.main_thread().as_mut().load(input)?;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn save<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    stream: *mut clap_ostream,
) -> bool
where
    P::MainThread: PluginStateImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        let output = OutputStream::from_raw_mut(&mut *stream);
        p.main_thread().as_mut().save(output)?;
        Ok(())
    })
    .is_some()
}
