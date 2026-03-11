use crate::audio_ports::AudioPortType;
use crate::configurable_audio_ports::{AudioPortRequestDetails, PortConfigDetails};
use crate::surround::SurroundChannel;
use core::slice;

/// Surround configuration data for a specific audio port.
///
/// This is an ordered list of channels, where each element of the list is a specific, named
/// [`SurroundChannel`].
///
/// See [`channel_count`](Self::channel_count) and [`get`](Self::get) to get the number of channels and
/// a channel at a specific index from the list, respectively.
#[derive(Debug, Copy, Clone)]
pub struct SurroundConfig<'a>(&'a [u8]);

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
    /// Returns the number of channels in this surround configuration.
    #[inline]
    pub fn channel_count(&self) -> usize {
        self.0.len()
    }

    /// Returns the specific [`SurroundChannel`] at the given channel index in this configuration.
    ///
    /// This returns `None` if the channel identifier at the given index is invalid or unknown, or
    /// if `index` is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<SurroundChannel> {
        SurroundChannel::from_raw(*self.0.get(index)?)
    }

    /// Wraps the given raw byte slice to interpret them as a surround configuration.
    #[inline]
    pub fn from_raw(raw: &'a [u8]) -> Self {
        Self(raw)
    }

    /// Returns this configuration data as a raw byte slice.
    #[inline]
    pub fn as_raw(&self) -> &'a [u8] {
        self.0
    }

    /// Returns this configuration as a generic [`AudioPortRequestDetails`](AudioPortRequestDetails).
    #[inline]
    pub fn as_request_details(&self) -> AudioPortRequestDetails<'a> {
        // SAFETY: This type ensures the slice pointer is valid for at least the given channel count,
        // and the details pointer type for SURROUND matches a [u8].
        unsafe {
            AudioPortRequestDetails::from_raw(
                Some(AudioPortType::SURROUND),
                self.0.len().try_into().unwrap_or(u32::MAX),
                self.0.as_ptr().cast(),
            )
        }
    }
}
