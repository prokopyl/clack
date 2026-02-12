use clack_common::process::AudioPortProcessingInfo;
use core::cell::Cell;

#[repr(C)]
pub(crate) struct CelledClapAudioBuffer {
    pub data32: *const *const f32,
    pub data64: *const *const f64,
    pub channel_count: u32,
    pub latency: u32,
    pub constant_mask: Cell<u64>, // Cell has the same memory layout as the inner type
}

// Statically assert that the two structs have the same memory representation
const _: () = {
    use clap_sys::audio_buffer::clap_audio_buffer;
    use core::mem::offset_of;
    assert!(size_of::<CelledClapAudioBuffer>() == size_of::<clap_audio_buffer>());
    assert!(align_of::<CelledClapAudioBuffer>() == align_of::<clap_audio_buffer>());
    assert!(offset_of!(CelledClapAudioBuffer, data32) == offset_of!(clap_audio_buffer, data32));
    assert!(offset_of!(CelledClapAudioBuffer, data64) == offset_of!(clap_audio_buffer, data64));
    assert!(
        offset_of!(CelledClapAudioBuffer, channel_count)
            == offset_of!(clap_audio_buffer, channel_count)
    );
    assert!(offset_of!(CelledClapAudioBuffer, latency) == offset_of!(clap_audio_buffer, latency));
    assert!(
        offset_of!(CelledClapAudioBuffer, constant_mask)
            == offset_of!(clap_audio_buffer, constant_mask)
    );
};

impl CelledClapAudioBuffer {
    #[inline]
    pub(crate) fn processing_info(&self) -> AudioPortProcessingInfo {
        // SAFETY: The shared reference "self" here guarantees the pointer is well-aligned and initialized.
        // This type also has the exact same memory layout as clap_audio_buffer, so it is safe to pass
        // a pointer to it to from_raw_ptr here.
        unsafe { AudioPortProcessingInfo::from_raw_ptr(self as *const _ as *const _) }
    }
}
