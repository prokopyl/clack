use crate::audio_ports::AudioPortType;
use crate::configurable_audio_ports::{AudioPortRequestDetails, PortConfigDetails};
use crate::surround::SurroundChannel;
use core::slice;

/// Ambisonic configuration data for an audio port.
#[derive(Debug, Copy, Clone)]
pub struct SurroundConfig<'a>(pub &'a [SurroundChannel]);

// SAFETY: AudioPortType::SURROUND is the identifier for the Surround port type.
unsafe impl<'a> PortConfigDetails<'a> for SurroundConfig<'a> {
    const PORT_TYPE: AudioPortType<'static> = AudioPortType::SURROUND;

    #[inline]
    unsafe fn from_details(raw: &AudioPortRequestDetails<'a>) -> Self {
        let len: usize = raw.channel_count().try_into().unwrap_or(usize::MAX);
        // SAFETY: Caller guarantees raw_details is valid matches CLAP_PORT_AMBISONIC,
        // which ensures the details pointer is of type [clap_ambisonic_config] with a len of
        // channel_count as per the CLAP spec
        let slice = unsafe { slice::from_raw_parts(raw.raw_details().cast(), len) };
        Self(slice)
    }
}

impl<'a> SurroundConfig<'a> {
    /// Returns this configuration as a generic [`AudioPortRequestDetails`](AudioPortRequestDetails),
    /// also using the provided `channel_count`.
    #[inline]
    pub fn as_request_details(&self) -> AudioPortRequestDetails<'a> {
        let channels = SurroundChannel::slice_as_raw(self.0);
        // SAFETY: This type ensures the slice pointer is valid for at least the given channel count,
        // and the details pointer type for SURROUND matches a [u8].
        unsafe {
            AudioPortRequestDetails::from_raw(
                Some(AudioPortType::SURROUND),
                self.0.len().try_into().unwrap_or(u32::MAX),
                channels.as_ptr().cast(),
            )
        }
    }
}

impl<'a> From<SurroundConfig<'a>> for &'a [SurroundChannel] {
    #[inline]
    fn from(value: SurroundConfig<'a>) -> Self {
        value.0
    }
}

impl<'a> From<&'a [SurroundChannel]> for SurroundConfig<'a> {
    #[inline]
    fn from(value: &'a [SurroundChannel]) -> Self {
        Self(value)
    }
}
