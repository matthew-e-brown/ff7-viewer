//! Extraction of archival/compressed formats, like [LGP][lgp] and [LGSS][lzss].

use std::fmt::Debug;

use thiserror::Error;


mod lgp;
mod lzss;

pub use lgp::*;
pub use lzss::*;


#[derive(Error, Debug)]
pub enum ParseError<'a> {
    #[error("expected a valid UTF-8 string")]
    Utf8Error(&'a [u8]),

    #[error("ran out of data while parsing")]
    EndOfBufferError,

    #[error("encountered invalid byte(s) at offset {1} of data")]
    InvalidValueError(&'a [u8], usize),

    #[error("encountered multiple files with the same name")]
    DuplicateNameError,

    #[error("encountered a file with no or an unknown file-type.")]
    UnknownFileTypeError,
}


/// Interprets a buffer as a null-terminated, ASCII string (a string-zero, or a `sz`). Also trims all null-bytes from
/// the buffers.
pub(crate) fn sz_to_str(data: &[u8]) -> Result<&str, ParseError> {
    let str = std::str::from_utf8(data).map_err(|_| ParseError::Utf8Error(data))?;
    let str = str.trim_end_matches('\0').trim_start_matches('\0');
    Ok(str)
}


/// Reads `len` bytes from the given buffer starting at `ptr`, then advances `ptr`. [`ParseError::EndOfDataError`] is
/// raised if `*ptr + len` exceeds the bounds of the buffer.
#[inline]
pub(crate) fn read<'a, 'b>(data: &'a [u8], ptr: &'b mut usize, len: usize) -> Result<&'a [u8], ParseError<'a>> {
    // Attempt to read and convert to the desired array size
    let res = data.get(*ptr..*ptr + len).ok_or(ParseError::EndOfBufferError)?;
    *ptr += len;
    Ok(res)
}


// --------------------------------------------------------------------------------------------------------------
// This entire section is temporary: as soon as `num_traits` adds `FromBytes`, this can be replaced with a single
// generic function. See https://github.com/rust-num/num-traits/pull/224.
// --------------------------------------------------------------------------------------------------------------

macro_rules! num_from_bytes {
    ($vis:vis, $func_name:ident, $num:ty, $method_name:ident, $doc_name:literal) => {
        #[doc="Generates"]
        #[doc=$doc_name]
        /// from bytes. Returns an [`EndOfBufferError`][ParseError::EndOfBufferError] if there are not enough bytes.
        #[allow(unused)]
        $vis fn $func_name(bytes: &[u8]) -> Result<$num, ParseError> {
            let bytes = bytes
                .get(0..std::mem::size_of::<$num>())   // just in case the buffer is longer, only read the first N bytes
                .ok_or(ParseError::EndOfBufferError)?; // if we couldn't read that many, we hit EOB
            Ok(<$num>::$method_name(bytes.try_into().unwrap()))
        }
    };
}

num_from_bytes!(pub(crate), u16_from_le_bytes, u16, from_le_bytes, "a `u16`");
num_from_bytes!(pub(crate), u32_from_le_bytes, u32, from_le_bytes, "a `u32`");
num_from_bytes!(pub(crate), u64_from_le_bytes, u64, from_le_bytes, "a `u64`");
num_from_bytes!(pub(crate), u128_from_le_bytes, u128, from_le_bytes, "a `u128`");
num_from_bytes!(pub(crate), usize_from_le_bytes, usize, from_le_bytes, "a `usize`");

num_from_bytes!(pub(crate), i8_from_le_bytes, i8, from_le_bytes, "an `i8`");
num_from_bytes!(pub(crate), i16_from_le_bytes, i16, from_le_bytes, "an `i16`");
num_from_bytes!(pub(crate), i32_from_le_bytes, i32, from_le_bytes, "an `i32`");
num_from_bytes!(pub(crate), i64_from_le_bytes, i64, from_le_bytes, "an `i64`");
num_from_bytes!(pub(crate), i128_from_le_bytes, i128, from_le_bytes, "an `i128`");
num_from_bytes!(pub(crate), isize_from_le_bytes, isize, from_le_bytes, "an `isize`");

num_from_bytes!(pub(crate), f32_from_le_bytes, f32, from_le_bytes, "an `f32`");
num_from_bytes!(pub(crate), f64_from_le_bytes, f64, from_le_bytes, "an `f64`");
