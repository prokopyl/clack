use super::*;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;

#[repr(C)]
pub struct IndexerWrapper<I> {
    inner: I,
    _no_send: PhantomData<*const ()>,
}

impl<I> IndexerWrapper<I> {
    #[inline]
    pub(crate) fn new(inner: I) -> Pin<Box<Self>> {
        Box::pin(IndexerWrapper {
            inner,
            _no_send: PhantomData,
        })
    }

    #[inline]
    pub(crate) fn as_raw_mut(self: Pin<&mut Self>) -> *mut c_void {
        // SAFETY: this method does not move anything out, it just gets the pointer
        let s = unsafe { self.get_unchecked_mut() };

        s as *mut Self as *mut c_void
    }
}
