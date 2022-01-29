use crate::factory::plugin::implementation;
use crate::factory::PluginFactories;
use crate::host::HostInfo;
use crate::plugin::wrapper::panic::catch_unwind;
use crate::plugin::{Plugin, PluginDescriptor, PluginInstance};
pub use clack_common::entry::PluginEntryDescriptor;
use clack_common::factory::plugin::PluginFactory;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;

pub trait PluginEntry: Sized {
    #[inline]
    fn init(_plugin_path: &CStr) -> bool {
        true
    }
    #[inline]
    fn de_init() {}

    fn declare_factories(builder: &mut PluginFactories);

    const DESCRIPTOR: PluginEntryDescriptor = PluginEntryDescriptor::new(clap_plugin_entry {
        clap_version: CLAP_VERSION,
        init: Some(init::<Self>),
        deinit: Some(de_init::<Self>),
        get_factory: Some(get_factory::<Self>),
    });
}

unsafe extern "C" fn init<E: PluginEntry>(plugin_path: *const ::std::os::raw::c_char) -> bool {
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
    .unwrap_or_else(|_| ::core::ptr::null())
}

pub struct SinglePluginEntry<P: for<'a> Plugin<'a>>(PhantomData<P>);

impl<P: for<'a> Plugin<'a>> PluginEntry for SinglePluginEntry<P> {
    #[inline]
    fn declare_factories(builder: &mut PluginFactories) {
        builder.register::<PluginFactory, Self>();
    }
}

impl<P: for<'a> Plugin<'a>> implementation::PluginFactory for SinglePluginEntry<P> {
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
    fn create_plugin<'p>(host_info: HostInfo<'p>, plugin_id: &[u8]) -> Option<PluginInstance<'p>> {
        if plugin_id == P::DESCRIPTOR.id().to_bytes_with_nul() {
            Some(PluginInstance::<'p>::new::<P>(host_info))
        } else {
            None
        }
    }
}
