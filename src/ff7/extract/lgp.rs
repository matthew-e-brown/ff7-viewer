//! Extracts [LGP files](https://wiki.ffrtt.ru/index.php/FF7/LGP_format).

use std::collections::HashMap;

use super::{sz_to_str, read, ExtractError};


/// The parsed contents of one LGP file.
pub struct LGPFile<'a> {
    /// The "creator" marker string from the file.
    ///
    /// Should always be either `"SQUARESOFT"` for official files and `"FICEDULA-LGP"` for patches made by Ficedula.
    /// Other values are not incorrect, just uncommon.
    pub creator: &'a str,

    /// The "terminator" marker string from the file.
    ///
    /// Should always be either `"FINAL FANTASY 7"` for official files and `"LGP-PATCH-FILE"` for patches. Other values
    /// are not incorrect, just uncommon.
    pub terminator: &'a str,

    /// All of the files that were found in this LGP archive. Keys are the filenames given to files in the archive and
    /// the values are the raw bytes, ready to be parsed further.
    pub files: HashMap<&'a str, &'a [u8]>,
}


impl<'a> LGPFile<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ExtractError> {
        let mut main_ptr = 0;
        let read = |p: &mut usize, l| read(data, p, l);

        // Check the first 12 bytes for the file's creator
        let creator = sz_to_str(read(&mut main_ptr, 12)?)?;
        if creator != "SQUARESOFT" && creator != "FICEDULA-LGP" {
            // log warning?
        }

        // Next is a 4-byte integer with the number of files from the archive. Can unwrap the `&[u8] to &[u8; 4]`
        // conversion because the success of `read` guarantees a correct length
        let file_count = u32::from_le_bytes(read(&mut main_ptr, 4)?.try_into().unwrap());

        // Next is the table of contents
        let mut files = HashMap::with_capacity(file_count as usize);
        let mut end_of_data = main_ptr; // updated as we look through the files pointed to by the TOC

        for _ in 0..file_count {
            let file_name_data = read(&mut main_ptr, 20)?;
            let file_name = sz_to_str(file_name_data)?;

            let offset = u32::from_le_bytes(read(&mut main_ptr, 4)?.try_into().unwrap());
            let check = u8::from_le_bytes(read(&mut main_ptr, 1)?.try_into().unwrap());
            let dupe = u16::from_le_bytes(read(&mut main_ptr, 2)?.try_into().unwrap());

            if check != 0x0E && check != 0x0B {
                // log warning?
            }

            if dupe != 0 {
                // handle duplicate
                return Err(ExtractError::DuplicateNameError);
            }

            // Go read the file's data
            // -----------------------

            let mut file_ptr = offset as usize;

            // verify that the TOC's name matches the actual file's name
            if sz_to_str(read(&mut file_ptr, 20)?)? != file_name {
                // log warning?
            }

            let file_size = u32::from_le_bytes(read(&mut file_ptr, 4)?.try_into().unwrap()) as usize;
            let file_data = read(&mut file_ptr, file_size)?;

            if let Some(_) = files.insert(file_name, file_data) {
                return Err(ExtractError::DuplicateNameError);
            }

            // Keep track of the furthest point we find in the file so that we can jump to the end later
            end_of_data = end_of_data.max(file_ptr);
        }

        // Finally there is a string, terminated by end of file
        let terminator = sz_to_str(&data[end_of_data..data.len()])?;
        Ok(Self { creator, terminator, files })
    }
}
