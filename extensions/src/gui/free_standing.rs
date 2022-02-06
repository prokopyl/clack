use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::gui_free_standing::{clap_plugin_gui_free_standing, CLAP_EXT_GUI_FREE_STANDING};

pub struct PluginFreeStandingGui {
    #[cfg_attr(not(feature = "clack-host"), allow(unused))]
    inner: clap_plugin_gui_free_standing,
}

unsafe impl Extension for PluginFreeStandingGui {
    const IDENTIFIER: &'static [u8] = CLAP_EXT_GUI_FREE_STANDING;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
impl PluginFreeStandingGui {
    #[inline]
    pub fn open(&self, plugin: &mut clack_host::plugin::PluginMainThread) -> bool {
        if let Some(open) = self.inner.open {
            unsafe { open(plugin.as_raw()) }
        } else {
            false
        }
    }
}

pub mod implementation {
    use clack_common::extensions::ExtensionImplementation;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::plugin::{Plugin, PluginError};
    use clap_sys::ext::gui_free_standing::clap_plugin_gui_free_standing;
    use clap_sys::plugin::clap_plugin;

    pub trait PluginFreeStandingGui {
        fn open(&mut self) -> Result<(), PluginError>;
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for super::PluginFreeStandingGui
    where
        P::MainThread: PluginFreeStandingGui,
    {
        const IMPLEMENTATION: &'static Self = &super::PluginFreeStandingGui {
            inner: clap_plugin_gui_free_standing {
                open: Some(open::<P>),
            },
        };
    }

    unsafe extern "C" fn open<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
    where
        P::MainThread: PluginFreeStandingGui,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin.main_thread().as_mut().open()?;
            Ok(())
        })
        .is_some()
    }
}
