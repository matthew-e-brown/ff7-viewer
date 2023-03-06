//! Extracts [LZSS files](https://wiki.ffrtt.ru/index.php/FF7/LZSS_format).

use super::{read, u32_from_le_bytes, ParseError};


/// Decompresses an LZSS archive.
///
/// See [module-level documentation](self) for more.
pub fn decompress_lzss(data: &[u8]) -> Result<Vec<u8>, ParseError> {
    let mut data_ptr = 0;
    let compressed_size = u32_from_le_bytes(read(data, &mut data_ptr, 4)?).unwrap() as usize;

    let mut buff = vec![0u8; 4096];
    let mut buff_ptr = 0xFEE;

    // We will need to expand this buffer, but since there's no way to know the decompressed size, this is a good start.
    let mut output = Vec::with_capacity(compressed_size);

    while data_ptr < data.len() {
        let ctrl_byte = read(data, &mut data_ptr, 1)?[0];

        for i in 0..8u8 {
            match (ctrl_byte >> i) & 1 {
                // Literal block (AKA, one byte)
                1 => {
                    let byte = read(data, &mut data_ptr, 1)?[0];
                    push_circular(&[byte], &mut buff, &mut buff_ptr); // push to reference buffer
                    output.push(byte); // push to output
                },
                // Reference block
                0 => {
                    // Read the two reference control bytes
                    // --------------------
                    let &[ ref_h, ref_l ] = read(data, &mut data_ptr, 2)? else {
                        // success of `read` with length 2 guarantees slice length
                        unreachable!();
                    };

                    let off = (ref_l as u16 & 0xF0 << 4) | (ref_h as u16);
                    let len = (ref_l as u16 & 0x0F) + 3;

                    // As `u16`, our control bytes look like:
                    //
                    // ref_h: ____ ____ OOOO OOOO
                    // ref_l: ____ ____ OOOO LLLL
                    //
                    // Hence the & and <<.

                    // Look into our circular buffer of already-read bytes and read them back
                    // --------------------

                    let mut data = get_circular(&buff, off as usize, len as usize);
                    push_circular(&data, &mut buff, &mut buff_ptr);
                    output.append(&mut data);
                },
                // anything `& 1` will always be 0 or 1
                _ => unreachable!(),
            }
        }
    }

    output.shrink_to(0); // make vec as small as possible just in-case we didn't get everything
    Ok(output)
}


fn push_circular(data: &[u8], buff: &mut [u8], ptr: &mut usize) {
    // go byte-by-byte so we can circle around when necessary
    for &byte in data {
        buff[*ptr % buff.len()] = byte;
        *ptr = (*ptr + 1) % buff.len();
    }
}


fn get_circular(buff: &[u8], off: usize, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(len);
    let a_end = (off + len).max(buff.len()); // read until end at most
    let b_end = (off + len) % buff.len(); // read from start up to the remaining amount
    v.extend_from_slice(&buff[off..a_end]);
    v.extend_from_slice(&buff[0..b_end]);
    v
}
