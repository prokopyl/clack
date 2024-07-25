#![allow(missing_docs)] // TODO

use std::cell::Cell;
use std::collections::Bound;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::ptr;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct AudioBuffer<'a, S> {
    ptr: NonNull<S>,
    len: usize,
    _lifetime: PhantomData<&'a [S]>,
}

impl<'a, S> AudioBuffer<'a, S> {
    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut S, len: usize) -> Self {
        let Some(ptr) = NonNull::new(ptr) else {
            null_audio_buffer()
        };

        Self {
            ptr,
            len,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn from_mut_slice(slice: &'a mut [S]) -> Self {
        Self {
            // SAFETY: pointer comes from a reference to a slice, it has to be non-null
            ptr: unsafe { NonNull::new_unchecked(slice.as_mut_ptr()) },
            len: slice.len(),
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            _lifetime: PhantomData,
        }
    }

    // from_slice_of_cells

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut S {
        self.ptr.as_ptr()
    }

    // TODO : test the heck outta this
    #[inline]
    pub fn slice_range(&self, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        if start >= self.len {
            return Self::empty();
        }

        // SAFETY: TODO
        let start_ptr = unsafe { self.ptr.as_ptr().add(start) };
        let len = self.len - start;

        let from_end = match range.end_bound() {
            Bound::Unbounded => 0,
            Bound::Included(i) | Bound::Excluded(i) if *i >= self.len => 0,
            Bound::Included(i) => self.len - *i,
            Bound::Excluded(i) => self.len - *i - 1,
        };

        let len = len - from_end;

        Self {
            // SAFETY: cannot be null TODO
            ptr: unsafe { NonNull::new_unchecked(start_ptr) },
            len,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_slice_of_cells(&self) -> &'a [Cell<S>] {
        // SAFETY: TODO
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len) }
    }

    #[inline]
    pub fn iter(&self) -> AudioBufferIter<'a, S> {
        let slice = self.as_slice_of_cells();
        AudioBufferIter {
            inner: slice.iter(),
        }
    }
}

impl<'a, S: Copy> AudioBuffer<'a, S> {
    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> S {
        // SAFETY: TODO
        unsafe { self.ptr.as_ptr().add(index).read() }
    }

    #[inline]
    pub fn get(&self, index: usize) -> S {
        if index >= self.len {
            out_of_bounds()
        }

        // SAFETY: we just checked index was in-bounds
        unsafe { self.get_unchecked(index) }
    }

    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn put_unchecked(&self, index: usize, value: S) {
        // SAFETY: TODO
        unsafe { self.ptr.as_ptr().add(index).write(value) }
    }

    #[inline]
    pub fn copy_to_slice(&self, buf: &mut [S]) {
        if buf.len() != self.len {
            slice_len_mismatch(self.len, buf.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy_nonoverlapping(self.ptr.as_ptr(), buf.as_mut_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_to_buffer(&self, buf: AudioBuffer<S>) {
        if buf.len != self.len {
            slice_len_mismatch(self.len, buf.len)
        }

        // SAFETY: TODO
        unsafe { ptr::copy(self.ptr.as_ptr(), buf.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_from_slice(&self, buf: &[S]) {
        if buf.len() != self.len {
            slice_len_mismatch(buf.len(), self.len)
        }

        // SAFETY: TODO
        unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), self.ptr.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn copy_from_buffer(&self, buf: AudioBuffer<S>) {
        if buf.len != self.len {
            slice_len_mismatch(buf.len, self.len)
        }

        // SAFETY: TODO
        unsafe { ptr::copy(buf.as_ptr(), self.ptr.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn fill(&self, _value: S) {
        todo!()
    }
}

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
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        for (a, b) in self.iter().zip(other) {
            if a.get() != b.get() {
                return false;
            }
        }

        true
    }
}

impl<'a, S: PartialEq + Copy> PartialEq<[S]> for AudioBuffer<'a, S> {
    fn eq(&self, other: &[S]) -> bool {
        if self.len != other.len() {
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
    fn eq(&self, other: &[S; N]) -> bool {
        if self.len != other.len() {
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

impl<'a, S: PartialEq + Copy> PartialEq<&[S]> for AudioBuffer<'a, S> {
    fn eq(&self, other: &&[S]) -> bool {
        if self.len != other.len() {
            return false;
        }

        for (a, b) in self.iter().zip(*other) {
            if a.get() != *b {
                return false;
            }
        }

        true
    }
}

impl<'a, S: PartialEq + Copy, const N: usize> PartialEq<&[S; N]> for AudioBuffer<'a, S> {
    fn eq(&self, other: &&[S; N]) -> bool {
        if self.len != other.len() {
            return false;
        }

        for (a, b) in self.iter().zip(*other) {
            if a.get() != *b {
                return false;
            }
        }

        true
    }
}

#[cold]
#[inline(never)]
fn out_of_bounds() -> ! {
    panic!("Out of bounds") // TODO: better error message
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
        "Buffer size mismatch: source has length {src_len}, but destination has length {dst_len}"
    )
}
