use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::events::{
    clap_event, clap_event_list, clap_event_transport, CLAP_TRANSPORT_IS_PLAYING,
};
use clap_sys::ext::params::{clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::process::clap_process;
use clap_sys::{host::clap_host, plugin::clap_plugin_entry, version::CLAP_VERSION};
use std::ffi::c_void;

use gain::clap_plugin_entry;

fn noop_event_list() -> clap_event_list {
    extern "C" fn size(_list: *const clap_event_list) -> u32 {
        0
    }
    extern "C" fn get(_list: *const clap_event_list, _index: u32) -> *const clap_event {
        ::core::ptr::null_mut()
    }
    extern "C" fn push_back(_list: *const clap_event_list, _event: *const clap_event) {}

    clap_event_list {
        ctx: core::ptr::null_mut(),
        size,
        get,
        push_back,
    }
}

#[test]
pub fn it_works() {
    extern "C" fn get_extension(_host: *const clap_host, _: *const i8) -> *const c_void {
        todo!()
    }
    extern "C" fn request_restart(_host: *const clap_host) {}
    extern "C" fn request_process(_host: *const clap_host) {}
    extern "C" fn request_callback(_host: *const clap_host) {}

    let host = clap_host {
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

    unsafe {
        let entry: &'static clap_plugin_entry = ::core::mem::transmute(&clap_plugin_entry);
        let desc = (entry.get_plugin_descriptor)(0);
        assert!(!desc.is_null());

        let plugin = (entry.create_plugin)(&host, (*desc).id).as_ref().unwrap();
        assert!((plugin.init)(plugin));
        (plugin.activate)(plugin, 44_100.0, 32, 32);
        assert!((plugin.start_processing)(plugin));

        // Params
        let params_ext = ((plugin.get_extension)(plugin, CLAP_EXT_PARAMS)
            as *const clap_plugin_params)
            .as_ref()
            .unwrap();

        assert_eq!((params_ext.count)(plugin), 0);

        // Process
        let inbuf = vec![69f32; 32];
        let outbuf = vec![0f32; 32];
        let in_chans = &[inbuf.as_ptr()];
        let out_chans = &[outbuf.as_ptr()];

        let ins = &[clap_audio_buffer {
            channel_count: 1,
            data32: in_chans.as_ptr(),
            latency: 0,
            constant_mask: 0,
            data64: ::core::ptr::null_mut(),
        }];

        let outs = &[clap_audio_buffer {
            channel_count: 1,
            data32: out_chans.as_ptr(),
            latency: 0,
            constant_mask: 0,
            data64: ::core::ptr::null_mut(),
        }];

        let transport = clap_event_transport {
            flags: CLAP_TRANSPORT_IS_PLAYING,

            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: 0.0,
            tempo_inc: 0.0,
            bar_start: 0,
            bar_number: 0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            tsig_num: 4,
            tsig_denom: 4,
        };
        let events = noop_event_list();

        let process = clap_process {
            steady_time: 0,
            frames_count: 32,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            audio_inputs: ins.as_ptr(),
            audio_outputs: outs.as_ptr(),

            transport: &transport,

            in_events: &events,
            out_events: &events,
        };

        (plugin.process)(plugin, &process);

        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 2.0)
        }

        (plugin.stop_processing)(plugin);
        // Done!
        (plugin.deactivate)(plugin);
        (plugin.destroy)(plugin);
    }
}
