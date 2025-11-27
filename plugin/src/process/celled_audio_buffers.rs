use clack_common::process::AudioPortProcessingInfo;
use clap_sys::audio_buffer::clap_audio_buffer;
use core::cell::Cell;

#[repr(C)]
pub(crate) struct CelledClapAudioBuffer {
    pub data32: *const *const f32,
    pub data64: *const *const f64,
    pub channel_count: u32,
    pub latency: u32,
    pub constant_mask: Cell<u64>, // Cell has the same memory layout as the inner type
}

impl CelledClapAudioBuffer {
    #[inline]
    pub(crate) fn as_raw_ptr(&self) -> *mut clap_audio_buffer {
        self as *const _ as *const _ as *mut _
    }

    #[inline]
    pub(crate) fn slice_as_raw_ptr(slice: &[CelledClapAudioBuffer]) -> *mut [clap_audio_buffer] {
        slice as *const _ as *const _ as *mut _
    }

    #[inline]
    pub(crate) fn from_raw_slice(slice: &mut [clap_audio_buffer]) -> &[Self] {
        // SAFETY: TODO
        unsafe { &*(slice as *mut [clap_audio_buffer] as *mut [CelledClapAudioBuffer]) }
    }

    #[inline]
    pub(crate) fn processing_info(&self) -> AudioPortProcessingInfo {
        // SAFETY: The shared reference "self" here guarantees the pointer is well-aligned and initialized.
        // This type also has the exact same memory layout as clap_audio_buffer, so it is safe to pass
        // a pointer to it to from_raw_ptr here.
        unsafe { AudioPortProcessingInfo::from_raw_ptr(self as *const _ as *const _) }
    }
}
