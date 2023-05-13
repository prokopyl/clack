//! Stream utilities.

use clap_sys::stream::{clap_istream, clap_ostream};
use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::io::{ErrorKind, Read, Write};
use std::marker::PhantomData;

/// An error code that can be raised by CLAP stream methods.
#[derive(Copy, Clone, Debug)]
pub struct StreamError {
    code: i64,
}

impl StreamError {
    /// Returns the underlying error code returned by the CLAP stream.
    pub fn code(&self) -> i64 {
        self.code
    }
}

impl Display for StreamError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clap stream error (code: {})", self.code)
    }
}

impl Error for StreamError {}

/// A CLAP data stream that can be read from.
///
/// This helper struct is designed to work with the standard [`Read`](Read) trait.
#[repr(C)]
pub struct InputStream<'a>(clap_istream, PhantomData<&'a clap_istream>);

impl<'a> InputStream<'a> {
    /// Creates a new input stream for an existing [reader](Read) implementation.
    pub fn from_reader<R: Read + Sized + 'a>(reader: &'a mut R) -> Self {
        Self(
            clap_istream {
                ctx: reader as *mut R as *mut _,
                read: Some(read::<R>),
            },
            PhantomData,
        )
    }

    /// Crates a new input stream for a C FFI-compatible pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the given `clap_istream` instance is valid.
    #[inline]
    pub unsafe fn from_raw_mut(raw: &mut clap_istream) -> &mut Self {
        &mut *(raw as *mut _ as *mut _)
    }

    /// Returns this input stream as a C FFI-compatible pointer.
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_istream {
        &mut self.0
    }
}

impl<'a> Read for InputStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let ret = if let Some(read) = self.0.read {
            unsafe { read(&self.0, buf.as_mut_ptr().cast(), buf.len() as u64) }
        } else {
            return Ok(0);
        };
        match ret {
            i if i >= 0 => Ok(i as usize),
            code => Err(std::io::Error::new(ErrorKind::Other, StreamError { code })),
        }
    }
}

/// A CLAP data stream that can be written to.
///
/// This helper struct is designed to work with the standard [`Write`](Write) trait.
#[repr(C)]
pub struct OutputStream<'a>(clap_ostream, PhantomData<&'a clap_ostream>);

impl<'a> OutputStream<'a> {
    /// Creates a new output stream for an existing [write](Write) implementation.
    pub fn from_writer<W: Write + Sized + 'a>(reader: &'a mut W) -> Self {
        Self(
            clap_ostream {
                ctx: reader as *mut W as *mut _,
                write: Some(write::<W>),
            },
            PhantomData,
        )
    }

    /// Crates a new output stream for a C FFI-compatible pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the given `clap_ostream` instance is valid.
    #[inline]
    pub unsafe fn from_raw_mut(raw: &mut clap_ostream) -> &mut Self {
        &mut *(raw as *mut _ as *mut _)
    }

    /// Returns this output stream as a C FFI-compatible pointer.
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_ostream {
        &mut self.0
    }
}

impl<'a> Write for OutputStream<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let ret = if let Some(write) = self.0.write {
            unsafe { write(&self.0, buf.as_ptr().cast(), buf.len() as u64) }
        } else {
            return Ok(0);
        };

        match ret {
            i if i >= 0 => Ok(i as usize),
            code => Err(std::io::Error::new(ErrorKind::Other, StreamError { code })),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

unsafe extern "C" fn read<R: Read + Sized>(
    istream: *const clap_istream,
    buffer: *mut c_void,
    size: u64,
) -> i64 {
    let reader = &mut *((*istream).ctx as *mut R);
    let buffer = core::slice::from_raw_parts_mut(buffer as *mut u8, size as usize);

    match handle_interrupted(|| reader.read(buffer)) {
        Ok(read) => read as i64,
        Err(_) => -1,
    }
}

unsafe extern "C" fn write<W: Write + Sized>(
    ostream: *const clap_ostream,
    buffer: *const c_void,
    size: u64,
) -> i64 {
    let writer = &mut *((*ostream).ctx as *mut W);
    let buffer = core::slice::from_raw_parts(buffer as *mut u8, size as usize);

    match handle_interrupted(|| writer.write(buffer)) {
        Ok(written) => written as i64,
        Err(_) => -1,
    }
}

fn handle_interrupted<F: FnMut() -> std::io::Result<usize>>(
    mut handler: F,
) -> std::io::Result<usize> {
    const MAX_ATTEMPTS: u8 = 5;
    let mut attempts = 0u8;

    loop {
        match handler() {
            Err(e) if e.kind() == ErrorKind::Interrupted && attempts < MAX_ATTEMPTS => {
                attempts += 1
            }
            res => return res,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn input_streams_work() {
        let src = b"Hello";
        let mut buf = vec![0; 5];
        let mut cursor = Cursor::new(src);

        let mut stream = InputStream::from_reader(&mut cursor);
        let res = stream.read(&mut buf).unwrap();
        assert_eq!(res, 5);
        assert_eq!(&buf, b"Hello");
    }

    #[test]
    fn output_streams_work() {
        let mut buf = vec![];

        let mut stream = OutputStream::from_writer(&mut buf);
        let res = stream.write(b"Hello").unwrap();

        assert_eq!(res, 5);
        assert_eq!(&buf, b"Hello");
    }
}
