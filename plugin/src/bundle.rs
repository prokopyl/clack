use crate::factory::plugin::implementation;
use crate::factory::PluginFactories;
use crate::host::HostInfo;
use crate::plugin::wrapper::panic::catch_unwind;
use crate::plugin::{Plugin, PluginDescriptor, PluginInstance};
use clack_common::factory::plugin::PluginFactory;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;

pub use clack_common::bundle::*;

pub trait PluginEntry: Sized {
    #[inline]
    fn init(_plugin_path: &CStr) -> bool {
        true
    }
    #[inline]
    fn de_init() {}

    fn declare_factories(builder: &mut PluginFactories);

    const DESCRIPTOR: PluginEntryDescriptor = PluginEntryDescriptor {
        clap_version: CLAP_VERSION,
        init: init::<Self>,
        deinit: de_init::<Self>,
        get_factory: get_factory::<Self>,
    };
}

unsafe extern "C" fn init<E: PluginEntry>(plugin_path: *const std::os::raw::c_char) -> bool {
    catch_unwind(|| E::init(CStr::from_ptr(plugin_path))).unwrap_or(false)
}

unsafe extern "C" fn de_init<E: PluginEntry>() {
    let _ = catch_unwind(|| E::de_init());
}

unsafe extern "C" fn get_factory<E: PluginEntry>(
    identifier: *const std::os::raw::c_char,
) -> *const c_void {
    catch_unwind(|| {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = PluginFactories::new(identifier);
        E::declare_factories(&mut builder);
        builder.found()
    })
    .unwrap_or(core::ptr::null())
}

pub struct SinglePluginEntry<'a, P: Plugin<'a>>(PhantomData<&'a P>);

impl<'a, P: Plugin<'a>> PluginEntry for SinglePluginEntry<'a, P> {
    #[inline]
    fn declare_factories(builder: &mut PluginFactories) {
        builder.register::<PluginFactory, Self>();
    }
}

impl<'a, P: Plugin<'a>> implementation::PluginFactory<'a> for SinglePluginEntry<'a, P> {
    #[inline]
    fn plugin_count() -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor> {
        match index {
            0 => Some(P::DESCRIPTOR),
            _ => None,
        }
    }

    #[inline]
    fn create_plugin(host_info: HostInfo<'a>, plugin_id: &[u8]) -> Option<PluginInstance<'a>> {
        if plugin_id == P::DESCRIPTOR.id().to_bytes_with_nul() {
            Some(PluginInstance::<'a>::new::<P>(host_info))
        } else {
            None
        }
    }
}
