#![allow(missing_docs)] // TODO

use core::cell::Cell;
use core::fmt::{Debug, Formatter};
use core::ops::Index;
use core::ptr;
use core::slice::SliceIndex;

#[repr(transparent)]
pub struct AudioBuffer<S> {
    inner: [Cell<S>],
}

impl<'a, S> AudioBuffer<S> {
    /// # Safety
    /// TODO
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut S, len: usize) -> &'a Self {
        if ptr.is_null() {
            null_audio_buffer()
        };

        Self::from_slice_of_cells(core::slice::from_raw_parts(ptr.cast(), len))
    }

    #[inline]
    pub fn from_mut_slice(slice: &'a mut [S]) -> &'a Self {
        Self::from_slice_of_cells(Cell::from_mut(slice).as_slice_of_cells())
    }

    #[inline]
    pub const fn from_slice_of_cells(slice: &'a [Cell<S>]) -> &'a Self {
        // SAFETY: TODO (omg)
        unsafe { core::mem::transmute(slice) }
    }

    #[inline]
    pub const fn empty() -> &'static Self {
        Self::from_slice_of_cells(&[])
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
    pub fn as_slice_of_cells(&'a self) -> &'a [Cell<S>] {
        &self.inner
    }

    #[inline]
    pub fn iter(&'a self) -> AudioBufferIter<'a, S> {
        AudioBufferIter {
            inner: self.inner.iter(),
        }
    }

    #[inline]
    fn reslice<I: SliceIndex<[Cell<S>], Output = [Cell<S>]>>(&self, index: I) -> &Self {
        AudioBuffer::from_slice_of_cells(self.inner.get(index).unwrap_or(&[]))
    }
}

impl<S: Copy> AudioBuffer<S> {
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
    pub fn copy_to_buffer(&self, buf: &AudioBuffer<S>) {
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
    pub fn copy_from_buffer(&self, buf: &AudioBuffer<S>) {
        if buf.len() != self.len() {
            slice_len_mismatch(buf.len(), self.len())
        }

        // SAFETY: TODO
        unsafe { ptr::copy(buf.as_ptr(), self.as_ptr(), buf.len()) }
    }

    #[inline]
    pub fn fill(&self, value: S) {
        for i in &self.inner {
            i.set(value)
        }
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

impl<'a, S> IntoIterator for &'a AudioBuffer<S> {
    type Item = &'a Cell<S>;
    type IntoIter = AudioBufferIter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<S> Index<usize> for AudioBuffer<S> {
    type Output = Cell<S>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl<S> Index<core::ops::RangeFull> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<core::ops::Range<usize>> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<core::ops::RangeFrom<usize>> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::RangeFrom<usize>) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<core::ops::RangeTo<usize>> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::RangeTo<usize>) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<core::ops::RangeInclusive<usize>> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::RangeInclusive<usize>) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<core::ops::RangeToInclusive<usize>> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: core::ops::RangeToInclusive<usize>) -> &Self::Output {
        self.reslice(index)
    }
}
impl<S> Index<(core::ops::Bound<usize>, core::ops::Bound<usize>)> for AudioBuffer<S> {
    type Output = Self;

    #[inline]
    fn index(&self, index: (core::ops::Bound<usize>, core::ops::Bound<usize>)) -> &Self::Output {
        self.reslice(index)
    }
}

impl<S: Debug + Copy> Debug for AudioBuffer<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_list();
        for s in self {
            dbg.entry(&s.get());
        }
        dbg.finish()
    }
}

impl<S: PartialEq + Copy> PartialEq for AudioBuffer<S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<S: PartialEq + Copy> PartialEq<[S]> for AudioBuffer<S> {
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

impl<S: PartialEq + Copy, const N: usize> PartialEq<[S; N]> for AudioBuffer<S> {
    #[inline]
    fn eq(&self, other: &[S; N]) -> bool {
        PartialEq::<[S]>::eq(self, other)
    }
}

impl<S: PartialEq + Copy> PartialEq<&[S]> for AudioBuffer<S> {
    #[inline]
    fn eq(&self, other: &&[S]) -> bool {
        PartialEq::<[S]>::eq(self, other)
    }
}

impl<S: PartialEq + Copy, const N: usize> PartialEq<&[S; N]> for AudioBuffer<S> {
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
