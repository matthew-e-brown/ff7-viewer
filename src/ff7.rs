use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;


pub trait ModelFileMarker {}


#[derive(Debug)]
pub struct HierarchyFile {

}

#[derive(Debug)]
pub struct ResourceFile {

}

#[derive(Debug)]
pub struct PolygonFile {

}

#[derive(Debug)]
pub struct TextureFile {

}

#[derive(Debug)]
pub struct AnimationFile {

}

impl ModelFileMarker for HierarchyFile {}
impl ModelFileMarker for ResourceFile {}
impl ModelFileMarker for PolygonFile {}
impl ModelFileMarker for TextureFile {}
impl ModelFileMarker for AnimationFile {}


/// The data that makes up a [ModelFile].
#[derive(Debug)]
pub enum ModelData<T: ModelFileMarker> {
    /// A serialized file; raw bytes as read from the file system.
    Raw(Vec<u8>),
    /// The deserialized file.
    Parsed(T),
}


/// One of the files that represent a Final Fantasy 7 model.
#[derive(Debug)]
pub enum ModelFile {
    /// An [.HRC file](https://wiki.ffrtt.ru/index.php/PSX/HRC). Represents a skeletal hierarchy. ASCII plaintext.
    Hierarchy(ModelData<HierarchyFile>),
    /// An [.RSD file](https://wiki.ffrtt.ru/index.php/PSX/RSD). References the other formats. ASCII plaintext.
    Resources(ModelData<ResourceFile>),
    /// A [.P file](https://wiki.ffrtt.ru/index.php/FF7/P). Contains the "compiled" .PLY, .MAT, and .GRP files. Unique
    /// to the PC version of FF7. Raw binary data of a special format.
    Polygons(ModelData<PolygonFile>),
    /// A [.TEX file](https://wiki.ffrtt.ru/index.php/FF7/TEX_format). Contains texture data. Raw binary data of a
    /// special format.
    Textures(ModelData<TextureFile>),
    /// An .A file. Contains animation data. There does not seem to be any documentation on this file format.
    Animation(ModelData<AnimationFile>),
}

impl ModelFile {
    /// Checks a file's name to determine the correct file type.
    pub fn new(name: &str, data: Vec<u8>) -> Result<Self, DecompressError> {
        match Path::new(name).extension() {
            Some(ext) => match ext.to_ascii_uppercase().to_str() {
                Some("HRC") => Ok(Self::Hierarchy(ModelData::Raw(data))),
                Some("RSD") => Ok(Self::Resources(ModelData::Raw(data))),
                Some("TEX") => Ok(Self::Textures(ModelData::Raw(data))),
                Some("A") => Ok(Self::Animation(ModelData::Raw(data))),
                Some("P") => Ok(Self::Polygons(ModelData::Raw(data))),
                Some(_) => Err(DecompressError::UnknownType),
                None => Err(DecompressError::Utf8Error),
            },
            None => Err(DecompressError::UnknownType),
        }
    }
}


/// Reads a string until the first 0x00 byte and returns the UTF-8 string up to that point.
fn null_terminated_string(buffer: &[u8]) -> Result<String, std::str::Utf8Error> {
    // Assume end of chunk if no null is found
    let i = buffer
        .iter()
        .position(|c| *c as char == '\0')
        .unwrap_or(buffer.len() - 1);
    Ok(std::str::from_utf8(&buffer[0..i])?.to_owned())
}


/// Represents all of the ways that an LGP archive's decompression could fail.
#[derive(Debug)]
pub enum DecompressError {
    /// A read or seek on the provided reader failed.
    ReadError(std::io::Error),
    /// A filename that should have been valid UTF-8 was not valid.
    Utf8Error,
    /// A file with no or an unknown file extension was encountered. Valid file types are listed in [LgpFile].
    UnknownType,
    /// More than one file with the same name was found.
    DuplicateFile,
}

impl From<std::io::Error> for DecompressError {
    fn from(err: std::io::Error) -> Self {
        Self::ReadError(err)
    }
}

impl From<std::str::Utf8Error> for DecompressError {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}

impl std::fmt::Display for DecompressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecompressError::Utf8Error => f.write_str("A filename in the archive was found to be invalid UTF-8."),
            DecompressError::UnknownType => f.write_str("A file in the archive was of an unknown type."),
            DecompressError::DuplicateFile => {
                f.write_str("There are some filenames in the archive that appear more than once.")
            },
            DecompressError::ReadError(inner) => {
                f.write_fmt(format_args!("Something went wrong reading the file: {inner}"))
            },
        }
    }
}


/// Takes any stream of bytes and decompresses it using the LGP format. Follows the format outlined on the
/// [QhimmWiki](https://wiki.ffrtt.ru/index.php/FF7/LGP_format) for decoding an LGP archive.
///
/// Returns a hashmap where the keys are filenames and the values are the files' data.
pub fn decompress_lgp<T: Read + Seek>(raw_data: &mut T) -> Result<HashMap<String, ModelFile>, DecompressError> {
    let mut buff = [0u8; 20]; // the most we'll ever need at once is 20 bytes for the file names

    // Section #1 -- File Header
    // -------------------------

    log::info!("Reading section #1, file header...");

    // First 12 bytes
    raw_data.read_exact(&mut buff[0..12])?;

    let cname = std::str::from_utf8(&buff[0..12])?;
    let cname = cname.trim_start_matches('\0'); // file "creator" is right-aligned

    log::trace!("Archive author is {}", cname);

    if cname != "SQUARESOFT" && cname != "FICEDULA-LGP" {
        log::warn!("LGP archive has abnormal file author, usually 'SQUARESOFT' or 'FICEDULA-LGP'...");
    }

    // Next is a four byte integer saying how many files the archive contains
    raw_data.read_exact(&mut buff[0..4])?;
    let file_count = u32::from_le_bytes(buff[0..4].try_into().unwrap());

    log::info!("\tArchive has {} files", file_count);
    log::info!("\tReading table of contents...");

    // Following is the table of contents (one entry per file, just stores offsets)
    let mut toc = HashMap::new();
    for i in 1..=file_count {
        log::trace!("Entry #{} in TOC", i);

        raw_data.read_exact(&mut buff[0..20])?;
        let filename = null_terminated_string(&buff[0..20])?.to_ascii_uppercase();

        log::trace!("\tFilename = {}", filename);

        raw_data.read_exact(&mut buff[0..4])?;
        let file_offset = u32::from_le_bytes(buff[0..4].try_into().unwrap());

        log::trace!("\tFile offset = 0x{:x}", file_offset);

        raw_data.read_exact(&mut buff[0..1])?;
        let check = buff[0];

        log::trace!("\tFile check code = {}", check);

        if check != 0x0E && check != 0x0B {
            log::warn!(
                "File {filename} in LGP archive has abnormal check code {}, usually 0x0E or 0x0B...",
                check
            );
        }

        raw_data.read_exact(&mut buff[0..2])?;
        let conflict = u16::from_le_bytes(buff[0..2].try_into().unwrap());

        log::trace!("\tFile conflict code = {}", conflict);

        if conflict != 0x00 {
            log::warn!(
                "File {filename} in LGP archive has abnormal conflict code {}, usually 0x00...",
                conflict
            );
        }

        toc.insert(filename, file_offset);
    }

    // Section #2 -- The 'CRC Code'
    // ----------------------------

    log::info!("\"Reading\" section #2, the \"CRC code\" (skipping past it)...");

    // 30 sets of 30 entries, two 16-bit words each. We don't need to do anything here so we just skip forward
    let data_start = raw_data.seek(SeekFrom::Current(30 * 30 * 2 * 2))?;

    // Section #3 -- Actual Data
    // -------------------------

    log::info!("Reading section #3, actual data...");

    // While we're at it, keep track of the furthest pointer we reach. We will use that to skip right to the end once
    // we're finished.
    let mut end_of_data: u64 = data_start;
    let mut all_files = HashMap::new();

    for (entry_name, offset) in toc {
        log::trace!("Reading file data for {}", entry_name);

        // Skip to entry and read file name
        raw_data.seek(SeekFrom::Start(offset as u64))?;
        raw_data.read_exact(&mut buff[0..20])?;

        let name = null_terminated_string(&buff[0..20])?.to_ascii_uppercase();

        log::trace!("\tRead filename {}", name);

        if *entry_name != name {
            log::warn!(
                "File header at offset {} of LGP archive does not agree on its name with with TOC: {} != {}...",
                offset,
                name,
                entry_name
            );
        }

        raw_data.read_exact(&mut buff[0..4])?;
        let len = u32::from_le_bytes(buff[0..4].try_into().unwrap());

        log::trace!("\tFile data is {} bytes long", len);

        let mut file_data = vec![0u8; len as usize];
        raw_data.read_exact(&mut file_data)?;

        let new_file = ModelFile::new(&name, file_data)?;
        if let Some(_) = all_files.insert(entry_name, new_file) {
            return Err(DecompressError::DuplicateFile);
        }

        let cur_ptr = raw_data.seek(SeekFrom::Current(0))?;
        if cur_ptr > end_of_data {
            end_of_data = cur_ptr;
            log::trace!("New 'furthest pointer', {:x}", end_of_data);
        }
    }

    // Section #4 -- Terminator
    // ------------------------

    log::info!("Reading section #4, terminator...");

    // Should just be 'FINAL FANTASY7' or 'LGP PATCH FILE' in all cases.
    let mut final_buffer = Vec::with_capacity(16);
    raw_data.seek(SeekFrom::Start(end_of_data))?;
    raw_data.read_to_end(&mut final_buffer)?; // read the rest of the file

    log::trace!("Final chunk of file: {:x?}", final_buffer);

    let term = String::from_utf8(final_buffer).unwrap_or("".to_owned());

    if term != "FINAL FANTASY7" && term != "LGP PATCH FILE" {
        log::warn!(
            "LGP archive has unusual terminator {}, usually 'FINAL FANTASY7' or 'LGP PATCH FILE'...",
            term
        );
    }

    log::info!("Finished parsing.");
    Ok(all_files)
}
