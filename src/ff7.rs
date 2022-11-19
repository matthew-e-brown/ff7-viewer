use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;


#[derive(Debug, Clone, Copy)]
struct TocEntry {
    pub file_offset: u32,
    pub check_code: u8,
    pub conflict_code: u16,
}


fn null_terminated_string(buffer: &[u8]) -> Result<String, std::str::Utf8Error> {
    // Assume end of chunk if no null is found
    let i = buffer
        .iter()
        .position(|c| *c as char == '\0')
        .unwrap_or(buffer.len() - 1);
    Ok(std::str::from_utf8(&buffer[0..i])?.to_owned())
}


const TEMP_OUTPUT: &str = "./temp-output";


/// Follows the format outlined on the [QhimmWiki](https://wiki.ffrtt.ru/index.php/FF7/LGP_format) for decoding an LGP
/// archive.
pub fn decompress_lgp(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This function is a temp implementation to test logic, will separate into a raw function that accepts a byte
    // iterator or something. The output directory is also temporary, eventually we will keep only the files we want.
    fs::create_dir_all(TEMP_OUTPUT)?;

    let mut buff = [0u8; 20]; // the most we'll ever need at once is 20 bytes for the file names
    let mut file = File::open(path)?;

    // Section #1 -- File Header
    // -------------------------

    // First 12 bytes
    file.read_exact(&mut buff[0..12])?;

    let cname = std::str::from_utf8(&buff[0..12])?;
    let cname = cname.trim_start_matches('\0'); // file "creator" is right-aligned

    if cname != "SQUARESOFT" && cname != "FICEDULA-LGP" {
        println!("LGP archive has abnormal file author, usually 'SQUARESOFT' or 'FICEDULA-LGP'...");
    }

    // Next is a four byte integer saying how many files the archive contains
    file.read_exact(&mut buff[0..4])?;
    let file_count = u32::from_le_bytes(buff[0..4].try_into().unwrap());

    // Following is the table of contents (one entry per file)
    let mut toc = HashMap::new();
    for _ in 0..file_count {
        file.read_exact(&mut buff[0..20])?;
        let filename = null_terminated_string(&buff[0..20])?.to_ascii_lowercase();

        file.read_exact(&mut buff[0..4])?;
        let file_offset = u32::from_le_bytes(buff[0..4].try_into().unwrap());

        file.read_exact(&mut buff[0..1])?;
        let check_code = buff[0];

        if check_code != 0x0E && check_code != 0x0B {
            println!(
                "File {filename} in LGP archive has abnormal check code {}, usually 0x0E or 0x0B...",
                check_code
            );
        }

        file.read_exact(&mut buff[0..2])?;
        let conflict_code = u16::from_le_bytes(buff[0..2].try_into().unwrap());

        if conflict_code != 0x00 {
            println!(
                "File {filename} in LGP archive has abnormal conflict code {}, usually 0x00...",
                conflict_code
            );
        }

        let entry = TocEntry {
            file_offset,
            check_code,
            conflict_code,
        };

        toc.insert(filename, entry);
    }

    // Section #2 -- The 'CRC Code'
    // ----------------------------

    // 30 sets of 30 entries, two 16-bit words each. We don't need to do anything here so we just skip forward
    let data_start = file.seek(SeekFrom::Current(30 * 30 * 2 * 2))?;

    // Section #3 -- Actual Data
    // -------------------------

    // While we're at it, keep track of the furthest pointer we reach. We will use that to skip right to the end once
    // we're finished.
    let mut end_of_data: u64 = data_start;
    for (entry_name, entry) in &toc {
        // Skip to entry and read file name
        file.seek(SeekFrom::Start(entry.file_offset as u64))?;
        file.read_exact(&mut buff[0..20])?;

        let name = null_terminated_string(&buff[0..20])?.to_ascii_lowercase();

        if *entry_name != name {
            println!(
                "File header at offset {} of LGP archive does not agree on its name with with TOC: {} != {}...",
                entry.file_offset, name, entry_name
            );
        }

        file.read_exact(&mut buff[0..4])?;
        let len = u32::from_le_bytes(buff[0..4].try_into().unwrap());

        let mut file_data = vec![0u8; len as usize];
        file.read_exact(&mut file_data)?;

        let mut new_file = File::create(PathBuf::from(TEMP_OUTPUT).join(name))?;
        new_file.write_all(&file_data)?;

        let cur_ptr = file.seek(SeekFrom::Current(0))?;
        if cur_ptr > end_of_data {
            end_of_data = cur_ptr;
        }
    }

    // Section #4 -- Terminator
    // ------------------------

    // Should just be 'FINAL FANTASY7' or 'LGP PATCH FILE' in all cases.
    let mut final_buffer = Vec::with_capacity(16);
    file.seek(SeekFrom::Start(end_of_data))?;
    file.read_to_end(&mut final_buffer)?;

    let term = String::from_utf8(final_buffer).unwrap_or("".to_owned());

    if term != "FINAL FANTASY7" && term != "LGP PATCH FILE" {
        println!("LGP archive has unusual terminator {}, usually 'FINAL FANTASY7' or 'LGP PATCH FILE'...", term);
    }

    Ok(())
}
