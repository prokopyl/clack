use crate::gui::UiSize;
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::plugin::wrapper::PluginWrapper;
use clack_plugin::plugin::{Plugin, PluginError};
use clap_sys::ext::gui::clap_plugin_gui;
use clap_sys::plugin::clap_plugin;

#[allow(unused)]
pub trait PluginGui {
    fn create(&mut self) -> Result<(), PluginError>;
    fn destroy(&mut self);

    fn set_scale(&mut self, scale: f64) -> bool {
        false
    }
    fn get_size(&mut self) -> Result<UiSize, PluginError>;
    fn can_resize(&mut self) -> bool;
    fn round_size(&mut self, size: UiSize) -> UiSize {
        size
    }
    fn set_size(&mut self, size: UiSize) -> bool;

    fn show(&mut self);
    fn hide(&mut self);
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for super::PluginGui
where
    P::MainThread: PluginGui,
{
    const IMPLEMENTATION: &'static Self = &super::PluginGui {
        inner: clap_plugin_gui {
            create: Some(create::<P>),
            destroy: Some(destroy::<P>),
            set_scale: Some(set_scale::<P>),
            get_size: Some(get_size::<P>),
            can_resize: Some(can_resize::<P>),
            round_size: Some(round_size::<P>),
            set_size: Some(set_size::<P>),
            show: Some(show::<P>),
            hide: Some(hide::<P>),
        },
    };
}

unsafe extern "C" fn create<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().create()?;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn destroy<'a, P: Plugin<'a>>(plugin: *const clap_plugin)
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().destroy();
        Ok(())
    });
}

unsafe extern "C" fn set_scale<'a, P: Plugin<'a>>(plugin: *const clap_plugin, scale: f64) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().set_scale(scale))
    })
    .unwrap_or(false)
}

unsafe extern "C" fn get_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = plugin.main_thread().as_mut().get_size()?;
        *width = size.width;
        *height = size.height;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn can_resize<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().can_resize())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn round_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = UiSize {
            width: *width,
            height: *height,
        };
        let rounded_size = plugin.main_thread().as_mut().round_size(size);
        *width = rounded_size.width;
        *height = rounded_size.height;
        Ok(())
    });
}

unsafe extern "C" fn set_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width: u32,
    height: u32,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = UiSize { width, height };
        Ok(plugin.main_thread().as_mut().set_size(size))
    })
    .is_some()
}

unsafe extern "C" fn show<'a, P: Plugin<'a>>(plugin: *const clap_plugin)
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().show();
        Ok(())
    });
}

unsafe extern "C" fn hide<'a, P: Plugin<'a>>(plugin: *const clap_plugin)
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().hide();
        Ok(())
    });
}
