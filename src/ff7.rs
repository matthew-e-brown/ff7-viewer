use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;


#[derive(Debug, Clone, Copy)]
struct TocEntry {
    pub file_offset: u32,
    pub _attribute_code: u8,
    pub _duplicate_check: u16,
}


/// Follows the format outlined on the [QhimmWiki](https://wiki.ffrtt.ru/index.php/FF7/LGP_format) for decoding an LGP
/// archive.
pub fn decompress_lgp(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This function is a temp implementation to test logic, will separate into a raw function that accepts a byte
    // iterator or something. The output directory is also temporary, eventually we will keep only the files we want.
    fs::create_dir_all("./temp-output")?;

    let mut buff = [0u8; 20];
    let mut file = File::open(path)?;

    // First 12 bytes
    file.read_exact(&mut buff[0..12])?;

    let cname = std::str::from_utf8(&buff[0..12])?;
    let cname = cname.trim_start_matches('\0'); // file "creator" is right-aligned

    if cname != "SQUARESOFT" && cname != "FICEDULA-LGP" {
        println!("Abnormal file author, usually 'SQUARESOFT' or 'FICEDULA-LGP'...");
    }

    // Next is a four byte integer saying how many files the archive contains
    file.read_exact(&mut buff[0..4])?;
    let file_count = u32::from_le_bytes(buff[0..4].try_into().unwrap());

    // Following is the table of contents (one entry per file)
    let mut toc = HashMap::new();
    for _ in 0..file_count {
        file.read_exact(&mut buff[0..20])?;
        let filename = {
            // Name is a null-terminated string, assume end of 20-byte chunk if no null is found
            let i = buff.iter().position(|c| *c as char == '\0').unwrap_or(buff.len() - 1);
            std::str::from_utf8(&buff[0..i])?.to_owned()
        };

        file.read_exact(&mut buff[0..4])?;
        let file_offset = u32::from_le_bytes(buff[0..4].try_into().unwrap());

        file.read_exact(&mut buff[0..1])?;
        let attr = buff[0];

        if attr != 0x0E && attr != 0x0B {
            println!("File {filename} in archive has abnormal attribute code {attr}, usually 0x0E or 0x0B...");
        }

        file.read_exact(&mut buff[0..2])?;
        let dupe = u16::from_le_bytes(buff[0..2].try_into().unwrap());

        if dupe != 0x00 {
            println!("File {filename} in archive has abnormal dupe-code {dupe}, usually 0x00...");
        }

        let entry = TocEntry {
            file_offset,
            _attribute_code: attr,
            _duplicate_check: dupe,
        };

        toc.insert(filename, entry);
    }

    // println!("{toc:#?}");

    Ok(())
}
