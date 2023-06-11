use clap_sys::audio_buffer::clap_audio_buffer;

pub struct OutputAudioPortInfo<'a> {
    pub(crate) buffer: &'a clap_audio_buffer, // TODO: split this properly
}

impl<'a> OutputAudioPortInfo<'a> {
    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.buffer.channel_count
    }

    #[inline]
    pub fn all_channels_constant(&self) -> bool {
        let all_constant_mask = (1 << self.channel_count()) - 1;
        (self.buffer.constant_mask & all_constant_mask) == all_constant_mask
    }

    #[inline]
    pub fn is_channel_constant(&self, channel_index: u32) -> bool {
        if channel_index > 31 {
            return false;
        }

        self.buffer.constant_mask & (1 << channel_index) != 0
    }

    // TODO: use ConstantMask
    #[inline]
    pub fn channel_constants(&self) -> impl Iterator<Item = bool> + '_ {
        (0..self.channel_count()).map(|c| self.is_channel_constant(c))
    }
}
