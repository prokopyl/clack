use crate::host::HostShared;
use clap_sys::host::clap_host;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_void, CString};
use std::marker::{PhantomData, PhantomPinned};
use std::os::raw::c_char;
use std::pin::Pin;
use std::sync::Arc;

use crate::entry::PluginEntry;
use basedrop::{Collector, Shared};

pub(crate) struct PluginInstanceCollector<'a> {
    collector: Option<Collector>,
    _lifetime: PhantomData<PluginEntry<'a>>,
}

pub(crate) struct PluginInstanceShared {
    _host_info: Arc<HostShared>,
    raw_host: clap_host,
    instance: *const clap_sys::plugin::clap_plugin,
    _pin: PhantomPinned,
}

unsafe impl Send for PluginInstanceShared {}
unsafe impl Sync for PluginInstanceShared {}

impl PluginInstanceShared {
    pub fn new<'a>(
        host_info: Arc<HostShared>,
        entry: &PluginEntry<'a>,
        plugin_id: &str,
    ) -> (Pin<Shared<Self>>, PluginInstanceCollector<'a>) {
        let mut raw_host = clap_host {
            clap_version: CLAP_VERSION,
            host_data: ::core::ptr::null_mut(),
            name: ::core::ptr::null_mut(),
            vendor: ::core::ptr::null_mut(),
            url: ::core::ptr::null_mut(),
            version: ::core::ptr::null_mut(),
            get_extension,
            request_restart,
            request_process,
            request_callback,
        };

        // SAFETY: host_info is guaranteed to outlive raw_host (thanks to Arc<>)
        unsafe {
            host_info.info().write_to_raw(&mut raw_host);
        }
        let collector = Collector::new();

        let mut arc = Shared::new(
            &collector.handle(),
            Self {
                instance: ::core::ptr::null(),
                _host_info: host_info,
                raw_host,
                _pin: PhantomPinned,
            },
        );

        let mutable = Shared::get_mut(&mut arc).unwrap();
        mutable.raw_host.host_data = mutable as *mut PluginInstanceShared as *mut c_void;
        mutable.instance = unsafe { Self::instantiate(entry, plugin_id, &mutable.raw_host) };

        (
            // SAFETY: Arc never moves its contents
            unsafe { Pin::new_unchecked(arc) },
            PluginInstanceCollector {
                collector: Some(collector),
                _lifetime: PhantomData,
            },
        )
    }

    unsafe fn instantiate<'a>(
        entry: &PluginEntry<'a>,
        plugin_id: &str,
        host_handle: *const clap_host,
    ) -> *const clap_sys::plugin::clap_plugin {
        let plugin_id = CString::new(plugin_id).unwrap();
        let instance = (entry.as_raw().create_plugin)(host_handle, plugin_id.as_ptr())
            .as_ref()
            .unwrap(); // TODO

        if !(instance.init)(instance) {
            panic!("Instanciation failed!"); // TODO
        }

        instance
    }

    #[inline]
    pub fn instance(&self) -> &clap_sys::plugin::clap_plugin {
        // SAFETY: the instance pointer is always valid as long as this isn't dropped.
        unsafe { &*self.instance }
    }
}

impl Drop for PluginInstanceShared {
    #[inline]
    fn drop(&mut self) {
        unsafe { ((*self.instance).destroy)(self.instance) }
    }
}

impl<'a> Drop for PluginInstanceCollector<'a> {
    fn drop(&mut self) {
        if let Some(mut collector) = self.collector.take() {
            collector.collect();
            if collector.try_cleanup().is_err() {
                // Can't do much but try to warn the developer.
                // TODO: maybe make this only in debug?
                eprintln!("PluginInstance could not be deallocated: it is likely because the audio processor side is still alive.")
            }
        } else {
            // Can't really do much here, can we
            panic!("PluginInstanceCollector destructor somehow called twice. This is a bug in clap-host, or worse.")
        }
    }
}

unsafe extern "C" fn get_extension(
    _host: *const clap_host,
    _extension_id: *const c_char,
) -> *const c_void {
    todo!()
}

unsafe extern "C" fn request_restart(_host: *const clap_host) {
    todo!()
}

unsafe extern "C" fn request_process(_host: *const clap_host) {
    todo!()
}

unsafe extern "C" fn request_callback(_host: *const clap_host) {
    todo!()
}
