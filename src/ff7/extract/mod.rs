//! Extraction of archival/compressed formats, like [LGP][lgp] and [LGSS][lzss].

use thiserror::Error;


pub mod lgp;
pub mod lzss;


#[derive(Error, Debug)]
pub enum ExtractError<'a> {
    #[error("expected a valid UTF-8 string")]
    Utf8Error(&'a [u8]),

    #[error("encountered end-of-file while parsing")]
    EofError,

    #[error("encountered multiple files with the same name")]
    DuplicateNameError,

    #[error("encountered a file with no or an unknown file-type.")]
    UnknownFileTypeError,
}


/// Reads `len` bytes from the given buffer starting at `ptr`, then advances `ptr`. [`ExtractError::EofError`] is raised
/// if `*ptr + len` exceeds the bounds of the buffer.
pub(crate) fn read<'a, 'b>(data: &'a [u8], ptr: &'b mut usize, len: usize) -> Result<&'a [u8], ExtractError<'a>> {
    // Attempt to read and convert to the desired array size
    let res = data.get(*ptr..*ptr + len).ok_or(ExtractError::EofError)?;
    *ptr += len;
    Ok(res)
}

/// Interprets a buffer as a null-terminated, ASCII string (a string-zero, or a `sz`). Also trims all null-bytes from
/// the buffers.
pub(crate) fn sz_to_str(data: &[u8]) -> Result<&str, ExtractError> {
    let str = std::str::from_utf8(data).map_err(|_| ExtractError::Utf8Error(data))?;
    let str = str.trim_end_matches('\0').trim_start_matches('\0');
    Ok(str)
}
