use clap_sys::audio_buffer::clap_audio_buffer;

struct HostAudioPortBuffer<B, S> {
    channel_buffers: Vec<B>,
    buffer_list: Vec<*const S>,
    min_buffer_length: usize,
}

impl<B: Sized + AsRef<[S]>, S> HostAudioPortBuffer<B, S> {
    pub fn new(channel_buffers: Vec<B>) -> Self {
        let buffer_list: Vec<_> = channel_buffers
            .iter()
            .map(|b| b.as_ref().as_ptr())
            .collect();

        let mut buf = Self {
            buffer_list,
            channel_buffers,
            min_buffer_length: 0,
        };

        buf.update_lengths();
        buf
    }

    fn update_lengths(&mut self) {
        self.min_buffer_length = self
            .channel_buffers
            .iter()
            .map(|b| b.as_ref().len())
            .min()
            .unwrap_or(0);
    }
}

impl<B: Sized + AsRef<[f32]>> HostAudioPortBuffer<B, f32> {
    // TODO: maybe unsafe?
    pub fn as_raw(&self) -> clap_audio_buffer {
        clap_audio_buffer {
            data32: self.buffer_list.as_ptr(),
            data64: ::core::ptr::null(),
            channel_count: self.buffer_list.len() as u32,
            latency: 0,       // TODO
            constant_mask: 0, // TODO
        }
    }
}

pub struct HostAudioBufferCollection<B, S> {
    ports: Vec<HostAudioPortBuffer<B, S>>,
    raw_ports: Vec<clap_audio_buffer>,
    min_buffer_length: usize,
}

impl<B, S> HostAudioBufferCollection<B, S> {
    #[inline]
    pub(crate) fn raw_buffers(&self) -> *const clap_audio_buffer {
        self.raw_ports.as_ptr()
    }

    #[inline]
    pub(crate) fn port_count(&self) -> usize {
        self.raw_ports.len()
    }

    #[inline]
    pub(crate) fn min_buffer_length(&self) -> usize {
        self.min_buffer_length
    }
}

impl<B: Sized + AsRef<[S]>, S> HostAudioBufferCollection<B, S> {
    pub fn get_channel_buffer(&self, port_index: usize, channel_index: usize) -> Option<&[S]> {
        Some(
            self.ports
                .get(port_index)?
                .channel_buffers
                .get(channel_index)?
                .as_ref(),
        )
    }
}

impl<B: Sized + AsRef<[f32]>> HostAudioBufferCollection<B, f32> {
    #[inline]
    pub fn for_ports_and_channels<F>(port_count: usize, channel_count: usize, buffer: F) -> Self
    where
        F: Fn() -> B,
    {
        Self::from_buffers((0..port_count).map(|_| (0..channel_count).map(|_| buffer())))
    }

    #[inline]
    pub fn from_buffers<IPorts, IChannels>(ports: IPorts) -> Self
    where
        IPorts: IntoIterator<Item = IChannels>,
        IChannels: IntoIterator<Item = B>,
    {
        let buffers = ports
            .into_iter()
            .map(|channels| HostAudioPortBuffer::new(channels.into_iter().collect()))
            .collect();
        Self::from_vecs(buffers)
    }

    fn from_vecs(ports: Vec<HostAudioPortBuffer<B, f32>>) -> Self {
        let raw_ports = ports.iter().map(|p| p.as_raw()).collect();

        Self {
            min_buffer_length: ports.iter().map(|p| p.min_buffer_length).min().unwrap_or(0),
            ports,
            raw_ports,
        }
    }
}
