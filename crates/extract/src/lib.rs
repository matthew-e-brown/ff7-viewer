use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum ParseError<'a> {
    #[error("expected a valid UTF-8 string")]
    Utf8Error(&'a [u8]),

    #[error("encountered end-of-file while parsing")]
    EofError,

    #[error("encountered multiple files with the same name")]
    DuplicateNameError,

    #[error("encountered a file with no or an unknown file-type.")]
    UnknownFileTypeError,
}


pub trait Parse<'a>: Sized {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError>;
}


/// An [.HRC file](https://wiki.ffrtt.ru/index.php/PSX/HRC). Represents a skeletal hierarchy. ASCII plaintext.
pub struct HierarchyFile<'a> {
    pub data: &'a [u8] // temporary
}

/// An [.RSD file](https://wiki.ffrtt.ru/index.php/PSX/RSD). References the other formats. ASCII plaintext.
pub struct ResourceFile<'a> {
    pub data: &'a [u8] // temporary
}

/// A [.P file](https://wiki.ffrtt.ru/index.php/FF7/P). Contains the "compiled" .PLY, .MAT, and .GRP files. Unique to
/// the PC version of FF7. Raw binary data of a special format.
pub struct PolygonFile<'a> {
    pub data: &'a [u8] // temporary
}

/// A [.TEX file](https://wiki.ffrtt.ru/index.php/FF7/TEX_format). Contains texture data. Raw binary data of a special
/// format.
pub struct TextureFile<'a> {
    pub data: &'a [u8] // temporary
}

/// An [.A file](https://wiki.ffrtt.ru/index.php/FF7/Battle/Battle_Animation_(PC). Contains animation data.
pub struct AnimationFile<'a> {
    pub data: &'a [u8] // temporary
}


pub enum File<'a> {
    HierarchyFile(HierarchyFile<'a>),
    ResourceFile(ResourceFile<'a>),
    PolygonFile(PolygonFile<'a>),
    TextureFile(TextureFile<'a>),
    AnimationFile(AnimationFile<'a>),
}

impl<'a> std::fmt::Debug for File<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HierarchyFile(_) => f.write_str("HierarchyFile"),
            Self::ResourceFile(_) => f.write_str("ResourceFile"),
            Self::PolygonFile(_) => f.write_str("PolygonFile"),
            Self::TextureFile(_) => f.write_str("TextureFile"),
            Self::AnimationFile(_) => f.write_str("AnimationFile"),
        }
    }
}


/// An [.LGP](https://wiki.ffrtt.ru/index.php/FF7/LGP_format) file.
#[derive(Debug)]
pub struct LGPFile<'a> {
    pub creator: &'a str,
    pub terminator: &'a str,
    pub files: HashMap<&'a str, File<'a>>,
}


/// Reads `len` bytes from the given buffer starting at `ptr`, then advances `ptr`.
fn read<'a, 'b>(data: &'a [u8], ptr: &'b mut usize, len: usize) -> Result<&'a [u8], ParseError<'a>> {
    // Attempt to read and convert to the desired array size
    let res = data.get(*ptr..*ptr + len).ok_or(ParseError::EofError)?;
    *ptr += len;
    Ok(res)
}

/// Interprets a chunk of bytes as a null-terminated string, including right-aligned ones.
fn null_str(data: &[u8]) -> Result<&str, ParseError> {
    let str = std::str::from_utf8(data).map_err(|_| ParseError::Utf8Error(data))?;
    let str = str
        .trim_end_matches('\0')
        .trim_start_matches('\0');
    Ok(str)
}


impl<'a> Parse<'a> for LGPFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        let mut main_ptr = 0;
        let read = |p: &mut usize, l| read(data, p, l);

        // Check the first 12 bytes for the file's creator
        let creator = null_str(read(&mut main_ptr, 12)?)?;
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
            let file_name = null_str(file_name_data)?;
            let file_ext = Path::new(file_name)
                .extension()
                .ok_or(ParseError::UnknownFileTypeError)?
                .to_str()
                .ok_or(ParseError::Utf8Error(file_name_data))?;

            let offset = u32::from_le_bytes(read(&mut main_ptr, 4)?.try_into().unwrap());
            let check = u8::from_le_bytes(read(&mut main_ptr, 1)?.try_into().unwrap());
            let dupe = u16::from_le_bytes(read(&mut main_ptr, 2)?.try_into().unwrap());

            if check != 0x0E && check != 0x0B {
                // log warning?
            }

            if dupe != 0 {
                // handle duplicate
                return Err(ParseError::DuplicateNameError);
            }

            // Go read the file's data
            // -----------------------

            let mut file_ptr = offset as usize;

            // verify that the TOC's name matches the actual file's name
            if null_str(read(&mut file_ptr, 20)?)? != file_name {
                // log warning?
            }

            let file_size = u32::from_le_bytes(read(&mut file_ptr, 4)?.try_into().unwrap()) as usize;
            let file_data = read(&mut file_ptr, file_size)?;
            let file = match file_ext {
                "HRC" | "hrc" => File::HierarchyFile(HierarchyFile::from_bytes(file_data)?),
                "RSD" | "rsd" => File::ResourceFile(ResourceFile::from_bytes(file_data)?),
                "TEX" | "tex" => File::TextureFile(TextureFile::from_bytes(file_data)?),
                "P" | "p" => File::PolygonFile(PolygonFile::from_bytes(file_data)?),
                "A" | "a" => File::AnimationFile(AnimationFile::from_bytes(file_data)?),
                _ => return Err(ParseError::UnknownFileTypeError),
            };

            if let Some(_) = files.insert(file_name, file) {
                return Err(ParseError::DuplicateNameError);
            }

            // Keep track of the furthest point we find in the file so that we can jump to the end later
            end_of_data = end_of_data.max(file_ptr);
        }

        // Finally there is a string, terminated by end of file
        let terminator = null_str(&data[end_of_data..data.len()])?;
        Ok(Self { creator, terminator, files })
    }
}


impl<'a> Parse<'a> for HierarchyFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self { data })
    }
}


impl<'a> Parse<'a> for ResourceFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self { data })
    }
}


impl<'a> Parse<'a> for PolygonFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self { data })
    }
}


impl<'a> Parse<'a> for TextureFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self { data })
    }
}


impl<'a> Parse<'a> for AnimationFile<'a> {
    fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(Self { data })
    }
}
