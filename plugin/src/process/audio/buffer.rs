#![allow(missing_docs)] // TODO

use std::cell::Cell;
use std::collections::Bound;
use std::fmt::{Debug, Formatter};
use std::ops::RangeBounds;
use std::ptr;

pub struct AudioBuffer<'a, S> {
    inner: &'a [Cell<S>],
}

impl<'a, S> AudioBuffer<'a, S> {
    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut S, len: usize) -> Self {
        if ptr.is_null() {
            null_audio_buffer()
        };

        Self {
            inner: core::slice::from_raw_parts(ptr.cast(), len),
        }
    }

    #[inline]
    pub fn from_mut_slice(slice: &'a mut [S]) -> Self {
        Self {
            inner: Cell::from_mut(slice).as_slice_of_cells(),
        }
    }

    #[inline]
    pub fn from_slice_of_cells(slice: &'a [Cell<S>]) -> Self {
        Self { inner: slice }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self { inner: &[] }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut S {
        self.inner.as_ptr().cast_mut().cast()
    }

    #[inline]
    pub fn slice_range(&self, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(i) => Bound::Included(*i),
            Bound::Excluded(i) => Bound::Excluded(*i),
            Bound::Unbounded => Bound::Unbounded,
        };

        let end = match range.end_bound() {
            Bound::Included(i) => Bound::Included(*i),
            Bound::Excluded(i) => Bound::Excluded(*i),
            Bound::Unbounded => Bound::Unbounded,
        };

        let slice = self.inner.get((start, end)).unwrap_or(&[]);

        Self { inner: slice }
    }

    #[inline]
    pub fn as_slice_of_cells(&self) -> &'a [Cell<S>] {
        self.inner
    }

    #[inline]
    pub fn iter(&self) -> AudioBufferIter<'a, S> {
        AudioBufferIter {
            inner: self.inner.iter(),
        }
    }
}

impl<'a, S: Copy> AudioBuffer<'a, S> {
    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> S {
        self.inner.get_unchecked(index).get()
    }

    #[inline]
    pub fn get(&self, index: usize) -> S {
        self.inner[index].get()
    }

    // TODO: try_get, try_put

    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn put_unchecked(&self, index: usize, value: S) {
        self.inner.get_unchecked(index).set(value)
    }

    #[inline]
    pub fn put(&self, index: usize, value: S) {
        self.inner[index].set(value)
    }

    #[inline]
    pub fn copy_to_slice(&self, buf: &mut [S]) {
        if buf.len() != self.len() {
            slice_len_mismatch(self.len(), buf.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_to_buffer(&self, buf: AudioBuffer<S>) {
        if buf.len() != self.len() {
            slice_len_mismatch(self.len(), buf.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy(self.as_ptr(), buf.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_from_slice(&self, buf: &[S]) {
        if buf.len() != self.len() {
            slice_len_mismatch(buf.len(), self.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), self.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_from_buffer(&self, buf: AudioBuffer<S>) {
        if buf.len() != self.len() {
            slice_len_mismatch(buf.len(), self.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy(buf.as_ptr(), self.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn fill(&self, value: S) {
        for i in self.inner {
            i.set(value)
        }
    }
}

impl<'a, S> Clone for AudioBuffer<'a, S> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, S> Copy for AudioBuffer<'a, S> {}

pub struct AudioBufferIter<'a, S> {
    inner: core::slice::Iter<'a, Cell<S>>,
}

impl<'a, S> Iterator for AudioBufferIter<'a, S> {
    type Item = &'a Cell<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, S> IntoIterator for AudioBuffer<'a, S> {
    type Item = &'a Cell<S>;
    type IntoIter = AudioBufferIter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, S> IntoIterator for &AudioBuffer<'a, S> {
    type Item = &'a Cell<S>;
    type IntoIter = AudioBufferIter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, S: Debug + Copy> Debug for AudioBuffer<'a, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_list();
        for s in self {
            dbg.entry(&s.get());
        }
        dbg.finish()
    }
}

impl<'a, S: PartialEq + Copy> PartialEq for AudioBuffer<'a, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(other.inner)
    }
}

impl<'a, S: PartialEq + Copy> PartialEq<[S]> for AudioBuffer<'a, S> {
    fn eq(&self, other: &[S]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (a, b) in self.iter().zip(other) {
            if a.get() != *b {
                return false;
            }
        }

        true
    }
}

impl<'a, S: PartialEq + Copy, const N: usize> PartialEq<[S; N]> for AudioBuffer<'a, S> {
    #[inline]
    fn eq(&self, other: &[S; N]) -> bool {
        PartialEq::<[S]>::eq(self, other)
    }
}

impl<'a, S: PartialEq + Copy> PartialEq<&[S]> for AudioBuffer<'a, S> {
    #[inline]
    fn eq(&self, other: &&[S]) -> bool {
        PartialEq::<[S]>::eq(self, other)
    }
}

impl<'a, S: PartialEq + Copy, const N: usize> PartialEq<&[S; N]> for AudioBuffer<'a, S> {
    #[inline]
    fn eq(&self, other: &&[S; N]) -> bool {
        PartialEq::<[S]>::eq(self, *other)
    }
}

#[cold]
#[inline(never)]
fn null_audio_buffer() -> ! {
    panic!("Invalid audio buffer: buffer pointer is NULL.")
}

#[cold]
#[inline(never)]
fn slice_len_mismatch(src_len: usize, dst_len: usize) -> ! {
    panic!(
        "Buffer size mismatch: source has length {}, but destination has length {}",
        src_len, dst_len
    )
}
