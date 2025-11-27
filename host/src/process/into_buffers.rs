use std::ptr::NonNull;

pub trait IntoPortsBuffers<'a> {
    type PortBuffer: PortBuffers<'a>;
    fn port_count(&self) -> usize;
    fn into_iterator(self) -> impl Iterator<Item = Self::PortBuffer>;
}

// TODO: unsafe
pub trait PortBuffers<'a> {
    type ChannelBuffer: ChannelBuffer<'a>;
    fn is_f64(&self) -> bool;
    fn latency(&self) -> u32;
    fn into_channels_refs(self) -> impl Iterator<Item = Self::ChannelBuffer>;
}

// TODO: unsafe
pub trait ChannelBuffer<'a>: 'a {
    const IS_F64: bool;
    fn buf_len(&self) -> usize;
    fn is_constant(&self) -> bool;
    fn buf_ptr(&mut self) -> *mut f32;
}

impl<'a, P> IntoPortsBuffers<'a> for P
where
    P: PortBuffers<'a>,
{
    type PortBuffer = P;

    fn port_count(&self) -> usize {
        1
    }

    fn into_iterator(self) -> impl Iterator<Item = Self::PortBuffer> {
        core::iter::once(self)
    }
}

impl<'a, P> IntoPortsBuffers<'a> for P
where
    P: IntoIterator<Item = ()>,
{
    type PortBuffer = P;

    fn port_count(&self) -> usize {
        1
    }

    fn into_iterator(self) -> impl Iterator<Item = Self::PortBuffer> {
        core::iter::once(self)
    }
}

impl<'a, P, const N: usize> IntoPortsBuffers<'a> for [P; N]
where
    P: PortBuffers<'a>,
{
    type PortBuffer = P;

    fn port_count(&self) -> usize {
        1
    }

    fn into_iterator(self) -> impl Iterator<Item = Self::PortBuffer> {
        self.into_iter()
    }
}

impl<'a, C: ChannelBuffer<'a>> PortBuffers<'a> for C {
    type ChannelBuffer = C;

    #[inline]
    fn is_f64(&self) -> bool {
        C::IS_F64
    }

    #[inline]
    fn latency(&self) -> u32 {
        0
    }

    #[inline]
    fn into_channels_refs(self) -> impl Iterator<Item = Self::ChannelBuffer> {
        core::iter::once(self)
    }
}

impl<'a, const N: usize, C: ChannelBuffer<'a>> PortBuffers<'a> for &'a mut [C; N] {
    type ChannelBuffer = &'a mut C;

    #[inline]
    fn is_f64(&self) -> bool {
        C::IS_F64
    }

    #[inline]
    fn latency(&self) -> u32 {
        0
    }

    #[inline]
    fn into_channels_refs(self) -> impl Iterator<Item = Self::ChannelBuffer> {
        self.iter_mut()
    }
}

impl<'a, C: ChannelBuffer<'a>> ChannelBuffer<'a> for &'a mut C {
    const IS_F64: bool = C::IS_F64;

    #[inline]
    fn buf_len(&self) -> usize {
        C::buf_len(self)
    }

    #[inline]
    fn is_constant(&self) -> bool {
        C::is_constant(self)
    }

    #[inline]
    fn buf_ptr(&mut self) -> *mut f32 {
        C::buf_ptr(self)
    }
}

impl<const N: usize> ChannelBuffer<'_> for [f32; N] {
    const IS_F64: bool = false;

    #[inline]
    fn buf_len(&self) -> usize {
        N
    }

    #[inline]
    fn is_constant(&self) -> bool {
        false
    }

    #[inline]
    fn buf_ptr(&mut self) -> *mut f32 {
        self.as_mut_ptr()
    }
}

impl<'a> ChannelBuffer<'a> for &'a mut [f32] {
    const IS_F64: bool = false;

    #[inline]
    fn buf_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_constant(&self) -> bool {
        false
    }

    #[inline]
    fn buf_ptr(&mut self) -> *mut f32 {
        self.as_mut_ptr()
    }
}

impl ChannelBuffer<'_> for () {
    const IS_F64: bool = false;

    #[inline]
    fn buf_len(&self) -> usize {
        0
    }

    #[inline]
    fn is_constant(&self) -> bool {
        true
    }

    #[inline]
    fn buf_ptr(&mut self) -> *mut f32 {
        NonNull::dangling().as_ptr()
    }
}
