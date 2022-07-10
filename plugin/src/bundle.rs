use crate::factory::plugin::{PluginFactory, PluginFactoryImpl};
use crate::factory::PluginFactories;
use crate::host::HostInfo;
use crate::plugin::descriptor::{PluginDescriptorWrapper, RawPluginDescriptor};
use crate::plugin::wrapper::panic::catch_unwind;
use crate::plugin::{Plugin, PluginInstance};
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::sync::Once;

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
        init: Some(init::<Self>),
        deinit: Some(de_init::<Self>),
        get_factory: Some(get_factory::<Self>),
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
        if identifier.is_null() {
            return core::ptr::null();
        }

        let identifier = CStr::from_ptr(identifier);
        let mut builder = PluginFactories::new(identifier);
        E::declare_factories(&mut builder);
        builder.found()
    })
    .unwrap_or(core::ptr::null())
}

pub struct SinglePluginEntry<'a, P: Plugin<'a>>(PhantomData<&'a P>);

static mut WRAPPER: Option<PluginDescriptorWrapper> = None;
static INIT: Once = Once::new();

fn get_wrapper<'a, P: Plugin<'a>>() -> Option<&'static PluginDescriptorWrapper> {
    INIT.call_once(|| {
        // SAFETY: this static is guaranteed to be initialized only once
        unsafe { WRAPPER = Some(PluginDescriptorWrapper::new(P::get_descriptor())) };
    });

    // SAFETY: this is only accessed and initialized by
    unsafe { WRAPPER.as_ref() }
}

impl<'a, P: Plugin<'a>> PluginEntry for SinglePluginEntry<'a, P> {
    fn init(_plugin_path: &CStr) -> bool {
        get_wrapper::<P>(); // Force initialization
        INIT.is_completed()
    }

    #[inline]
    fn declare_factories(builder: &mut PluginFactories) {
        builder.register::<PluginFactory, Self>();
    }
}

impl<'a, P: Plugin<'a>> PluginFactoryImpl<'a> for SinglePluginEntry<'a, P> {
    #[inline]
    fn plugin_count() -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(index: u32) -> Option<&'static RawPluginDescriptor> {
        match index {
            0 => Some(get_wrapper::<P>().unwrap().get_raw()),
            _ => None,
        }
    }

    #[inline]
    fn create_plugin(host_info: HostInfo<'a>, plugin_id: &CStr) -> Option<PluginInstance<'a>> {
        let descriptor = get_wrapper::<P>().unwrap();

        if plugin_id == descriptor.descriptor().id() {
            Some(PluginInstance::new::<P>(host_info, descriptor.get_raw()))
        } else {
            None
        }
    }
}
