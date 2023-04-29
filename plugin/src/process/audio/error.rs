use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BufferError {
    InvalidChannelBuffer,
    MismatchedBufferPair,
}

impl Display for BufferError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::InvalidChannelBuffer => {
                f.write_str("Invalid channel buffer: both the data32 and data64 pointers were null")
            },
            BufferError::MismatchedBufferPair => f.write_str("Invalid channel buffer pairing: attempted to read/write a 32-bit buffer and a 64-bit buffer together")
        }
    }
}

impl Error for BufferError {}
