#![allow(missing_docs)]
#![allow(clippy::undocumented_unsafe_blocks)]

use crate::prelude::InputAudioBuffers;
use clap_sys::audio_buffer::clap_audio_buffer;

pub struct AudioPortBuffers {
    port_buffers: Vec<clap_audio_buffer>,
}

impl AudioPortBuffers {
    pub fn new() -> Self {
        Self {
            port_buffers: Vec::new(),
        }
    }

    pub fn with_inputs<'a>(&mut self, ports: impl IntoPorts<'a>) -> InputAudioBuffers<'a> {
        todo!()
    }
}

pub struct InputAudioPort<T> {
    pub buffer: T,
}

pub unsafe trait IntoPorts<'a> {
    const X: bool;
    fn sample_count(&self) -> u32;
}

pub unsafe trait BorrowAudioPort<'a> {
    const IS_F64: bool;
    const WRITABLE: bool;

    fn sample_count(&self) -> u32;
    fn channel_count(&self) -> u32;
    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>);
}

pub unsafe trait BorrowAudioChannel<'a> {
    const IS_F64: bool;
    const WRITABLE: bool;

    fn sample_count(&self) -> u32;
    fn into_ptr(self) -> *mut f32;
}

unsafe impl<'a, const N: usize> BorrowAudioChannel<'a> for &'a [f32; N] {
    const IS_F64: bool = false;
    const WRITABLE: bool = false;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        N as u32
    }

    #[inline(always)]
    fn into_ptr(self) -> *mut f32 {
        self.as_ptr().cast_mut()
    }
}

unsafe impl<'a, const N: usize> BorrowAudioChannel<'a> for &'a mut [f32; N] {
    const IS_F64: bool = false;
    const WRITABLE: bool = true;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        N as u32
    }

    #[inline(always)]
    fn into_ptr(self) -> *mut f32 {
        self.as_ptr().cast_mut()
    }
}

unsafe impl<'a, const N: usize, const M: usize> BorrowAudioPort<'a> for &'a mut [[f32; N]; M] {
    const IS_F64: bool = false;
    const WRITABLE: bool = true;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        N as u32
    }

    fn channel_count(&self) -> u32 {
        M as u32
    }

    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.extend(self.iter_mut().map(|x| x.as_mut_ptr()));
    }
}

unsafe impl<'a, const N: usize, const M: usize> BorrowAudioPort<'a> for &'a [[f32; N]; M] {
    const IS_F64: bool = false;
    const WRITABLE: bool = false;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        N as u32
    }

    fn channel_count(&self) -> u32 {
        M as u32
    }

    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.extend(self.iter().map(|x| x.as_ptr().cast_mut()));
    }
}

unsafe impl<'a, C: BorrowAudioChannel<'a>> BorrowAudioPort<'a> for C {
    const IS_F64: bool = C::IS_F64;
    const WRITABLE: bool = C::WRITABLE;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        C::sample_count(self)
    }

    #[inline(always)]
    fn channel_count(&self) -> u32 {
        1
    }

    #[inline(always)]
    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.push(self.into_ptr());
    }
}

unsafe impl<'a, C: BorrowAudioChannel<'a>, const N: usize> BorrowAudioPort<'a> for [C; N] {
    const IS_F64: bool = C::IS_F64;
    const WRITABLE: bool = C::WRITABLE;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        self.iter().map(|x| x.sample_count()).min().unwrap_or(0)
    }

    #[inline(always)]
    fn channel_count(&self) -> u32 {
        N as u32
    }

    #[inline(always)]
    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.extend(self.into_iter().map(|b| b.into_ptr()))
    }
}

unsafe impl<'a, C: BorrowAudioChannel<'a> + Copy, const N: usize> BorrowAudioPort<'a> for &[C; N] {
    const IS_F64: bool = C::IS_F64;
    const WRITABLE: bool = C::WRITABLE;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        self.iter().map(|x| x.sample_count()).min().unwrap_or(0)
    }

    #[inline(always)]
    fn channel_count(&self) -> u32 {
        N as u32
    }

    #[inline(always)]
    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.extend(self.iter().map(|b| b.into_ptr()))
    }
}

unsafe impl<'a, C: BorrowAudioChannel<'a> + Copy> BorrowAudioPort<'a> for &[C] {
    const IS_F64: bool = C::IS_F64;
    const WRITABLE: bool = C::WRITABLE;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        self.iter().map(|x| x.sample_count()).min().unwrap_or(0)
    }

    #[inline(always)]
    fn channel_count(&self) -> u32 {
        self.len() as u32
    }

    #[inline(always)]
    fn into_channel_ptrs(self, buf: &mut Vec<*mut f32>) {
        buf.extend(self.iter().map(|b| b.into_ptr()))
    }
}

unsafe impl<'a, P: BorrowAudioPort<'a>> IntoPorts<'a> for P {
    const X: bool = false;

    #[inline(always)]
    fn sample_count(&self) -> u32 {
        P::sample_count(self)
    }
}

unsafe impl<'a, P: BorrowAudioPort<'a>, const N: usize> IntoPorts<'a> for [P; N] {
    const X: bool = false;

    fn sample_count(&self) -> u32 {
        todo!()
    }
}

#[inline]
fn to_raw_buffer<'a, P: BorrowAudioPort<'a>>(port: &P) -> clap_audio_buffer {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_channel() {
        let mut buf = [0.; 4];
        let mut ports = AudioPortBuffers::new();

        let _ = ports.with_inputs(&buf);
        let _ = ports.with_inputs(&mut buf);
        let _ = ports.with_inputs([&buf]);
        let _ = ports.with_inputs(&[&buf][..]);
    }

    #[test]
    fn stereo() {
        let mut buf = [[0.; 4]; 2];
        let mut ports = AudioPortBuffers::new();

        let _ = ports.with_inputs(&buf);
        let _ = ports.with_inputs(&mut buf);
        let _ = ports.with_inputs([&buf]);
        let _ = ports.with_inputs(&[&buf][..]);
    }
}
