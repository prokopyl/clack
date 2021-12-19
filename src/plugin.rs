use core::ffi::c_void;
use std::ffi::CStr;

use crate::extension::ExtensionDeclarations;
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_CONTINUE};
use clap_sys::{
    plugin::{clap_plugin, clap_plugin_descriptor, CLAP_PLUGIN_AUDIO_EFFECT},
    version::CLAP_VERSION,
};
use std::marker::PhantomData;

use crate::host::{HostHandle, HostInfo};
use crate::process::audio::Audio;
use crate::process::Process;

pub struct PluginInstance<'a> {
    inner: Box<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginInstance<'a> {
    #[inline]
    pub(crate) fn into_owned_ptr(self) -> *mut clap_plugin {
        Box::into_raw(self.inner)
    }

    fn get_plugin_desc<P: Plugin<'a>>(data: PluginData<'a, P>) -> clap_plugin {
        clap_plugin {
            desc: &P::DESCRIPTOR.0,
            plugin_data: Box::into_raw(Box::new(data)).cast(),
            init: Self::init::<P>,
            destroy: Self::destroy::<P>,
            activate: Self::activate::<P>,
            deactivate: Self::deactivate::<P>,
            start_processing: Self::start_processing::<P>,
            stop_processing: Self::stop_processing::<P>,
            process: Self::process::<P>,
            get_extension: Self::get_extension::<P>,
            on_main_thread: Self::on_main_thread::<P>,
        }
    }

    /// # Safety
    /// The plugin pointer must be valid
    pub unsafe fn get_plugin<P: Plugin<'a>>(plugin: *const clap_plugin) -> &'a P {
        let data = &mut *((*plugin).plugin_data as *mut PluginData<'a, P>);
        data.plugin_data
            .as_ref()
            .expect("Plugin is not initialized") // TODO: unsafe unwrap
    }

    pub fn new<P: Plugin<'a>>(host_info: HostInfo<'a>) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        let data = PluginData::<'a, P> {
            host,
            plugin_data: None,
        };
        Self {
            inner: Box::new(Self::get_plugin_desc(data)),
            lifetime: PhantomData,
        }
    }

    unsafe extern "C" fn init<P: Plugin<'a>>(plugin: *const clap_plugin) -> bool {
        let data = &mut *((*plugin).plugin_data as *mut PluginData<'a, P>);
        if data.plugin_data.is_some() {
            eprintln!("Plugin is already initialized");
            return false;
        }

        data.plugin_data = P::new(data.host);
        data.plugin_data.is_some()
    }

    unsafe extern "C" fn destroy<P: Plugin<'a>>(plugin: *const clap_plugin) {
        let plugin = Box::from_raw(plugin as *mut clap_plugin);

        if !plugin.plugin_data.is_null() {
            Box::from_raw(plugin.plugin_data.cast::<PluginData<'a, P>>());
        }
    }

    unsafe extern "C" fn activate<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        sample_rate: f64,
        min_frames_count: u32,
        max_frames_count: u32,
    ) {
        P::activate(
            Self::get_plugin(plugin),
            sample_rate,
            min_frames_count,
            max_frames_count,
        )
    }

    unsafe extern "C" fn deactivate<P: Plugin<'a>>(plugin: *const clap_plugin) {
        P::deactivate(Self::get_plugin(plugin))
    }

    unsafe extern "C" fn start_processing<P: Plugin<'a>>(plugin: *const clap_plugin) -> bool {
        P::start_processing(Self::get_plugin(plugin))
    }

    unsafe extern "C" fn stop_processing<P: Plugin<'a>>(plugin: *const clap_plugin) {
        P::stop_processing(Self::get_plugin(plugin))
    }

    unsafe extern "C" fn process<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        // SAFETY: process ptr is never accessed later, and is guaranteed to be valid and unique by the host
        let (process, audio) = Process::from_raw(process);
        P::process(Self::get_plugin(plugin), process, audio); // TODO: handle return status
        CLAP_PROCESS_CONTINUE
    }

    unsafe extern "C" fn get_extension<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        identifier: *const std::os::raw::c_char,
    ) -> *const c_void {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = ExtensionDeclarations::new(identifier);
        P::declare_extensions(Self::get_plugin(plugin), &mut builder);
        builder.found()
    }

    unsafe extern "C" fn on_main_thread<P: Plugin<'a>>(plugin: *const clap_plugin) {
        P::on_main_thread(Self::get_plugin(plugin))
    }
}

struct PluginData<'a, P: Plugin<'a>> {
    host: HostHandle<'a>,
    plugin_data: Option<P>,
}

pub trait Plugin<'a>: Sized + Send + Sync + 'a {
    const ID: &'static [u8]; // TODO: handle null-terminating stuff safely

    fn new(host: HostHandle<'a>) -> Option<Self>;
    #[inline]
    fn activate(&self, _sample_rate: f64, _min_sample_count: u32, _max_sample_count: u32) {}
    #[inline]
    fn deactivate(&self) {}

    #[inline]
    fn start_processing(&self) -> bool {
        true
    }
    #[inline]
    fn stop_processing(&self) {}

    fn process(&self, process: &Process, audio: Audio); // TODO: status

    #[inline]
    fn declare_extensions(&self, _builder: &mut ExtensionDeclarations<Self>) {}

    #[inline]
    fn on_main_thread(&self) {}

    const DESCRIPTOR: &'static PluginDescriptor = &PluginDescriptor(clap_plugin_descriptor {
        clap_version: CLAP_VERSION,
        id: Self::ID.as_ptr() as *const i8,
        name: EMPTY.as_ptr() as *const i8,
        vendor: EMPTY.as_ptr() as *const i8,
        url: EMPTY.as_ptr() as *const i8,
        manual_url: EMPTY.as_ptr() as *const i8,
        version: EMPTY.as_ptr() as *const i8,
        description: EMPTY.as_ptr() as *const i8,
        keywords: EMPTY.as_ptr() as *const i8,
        support_url: EMPTY.as_ptr() as *const i8,
        // FIXME: Why is this u64 but plugin types are i32?
        plugin_type: CLAP_PLUGIN_AUDIO_EFFECT as u64, // TODO
    });
}

pub struct PluginDescriptor(pub(crate) clap_plugin_descriptor);

const EMPTY: &[u8] = b"\0"; // TODO
