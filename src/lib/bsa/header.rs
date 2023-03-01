use std::io::{BufRead, Result, Write};

use bitflags::bitflags;

use crate::{io_error, ReadEx, WriteEx};

use super::Version;

pub const BSA_SIGNATURE: &[u8; 4] = b"BSA\0";

bitflags! {
    pub struct Flags: u32 {
        const HAS_FOLDER_NAMES         = 0b0000000000000001;
        const HAS_FILE_NAMES           = 0b0000000000000010;
        const COMPRESSED_BY_DEFAULT    = 0b0000000000000100;
        const RETAIN_FOLDER_NAMES      = 0b0000000000001000;
        const RETAIN_FILE_NAMES        = 0b0000000000010000;
        const RETAIN_FILE_NAME_OFFSETS = 0b0000000000100000; // since v104
        const XBOX                     = 0b0000000001000000;
        const RETAIN_STRINGS           = 0b0000000010000000;
        const EMBEDDED_FILE_NAMES      = 0b0000000100000000; // since v104
        const XMEM_CODEC               = 0b0000001000000000; // since v104
        const UNKNOWN_11               = 0b0000010000000000; // xbox Oblivion has it
    }

    #[derive(Default)]
    pub struct FileFlags: u16 {
        const MESHES                   = 0b0000000000000001;
        const TEXTURES                 = 0b0000000000000010;
        const MENUS                    = 0b0000000000000100;
        const SOUNDS                   = 0b0000000000001000;
        const VOICES                   = 0b0000000000010000;
        const SHADERS                  = 0b0000000000100000;
        const TREES                    = 0b0000000001000000;
        const FONTS                    = 0b0000000010000000;
        const MISC                     = 0b0000000100000000;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags::HAS_FOLDER_NAMES | Flags::HAS_FILE_NAMES
    }
}

pub struct Header {
    pub version: Version,
    pub folder_records_offset: u32,
    pub flags: Flags,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: FileFlags,
}

impl Header {
    pub fn embedded_file_names(&self) -> bool {
        (self.version == Version::V104 || self.version == Version::V105)
            && self.flags.contains(Flags::EMBEDDED_FILE_NAMES)
    }
}

impl Default for Header {
    fn default() -> Self {
        Header {
            version: Version::V105,
            folder_records_offset: 36,
            flags: Flags::default(),
            folder_count: 0,
            file_count: 0,
            total_folder_name_length: 0,
            total_file_name_length: 0,
            file_flags: FileFlags::default(),
        }
    }
}

pub trait ReadHeader: BufRead {
    #[inline]
    fn read_header(&mut self) -> Result<Header> {
        Ok(Header {
            version: Version::try_from(self.read_u32_le()?)
                .map_err(|err| io_error!(InvalidData, "{}", err))?,
            folder_records_offset: self.read_u32_le()?,
            flags: Flags::from_bits(self.read_u32_le()?)
                .ok_or_else(|| io_error!(InvalidData, "invalid flags"))?,
            folder_count: self.read_u32_le()?,
            file_count: self.read_u32_le()?,
            total_folder_name_length: self.read_u32_le()?,
            total_file_name_length: self.read_u32_le()?,
            file_flags: FileFlags::from_bits(self.read_u16_le()?)
                .ok_or_else(|| io_error!(InvalidData, "invalid file flags"))?,
        })
    }
}

impl<R: BufRead + ?Sized> ReadHeader for R {}

pub trait WriteHeader: Write {
    #[inline]
    fn write_header(&mut self, hdr: &Header) -> Result<()> {
        self.write_u32_le(u32::from(&hdr.version))?;
        self.write_u32_le(hdr.folder_records_offset)?;
        self.write_u32_le(hdr.flags.bits())?;
        self.write_u32_le(hdr.folder_count)?;
        self.write_u32_le(hdr.file_count)?;
        self.write_u32_le(hdr.total_folder_name_length)?;
        self.write_u32_le(hdr.total_file_name_length)?;
        self.write_u16_le(hdr.file_flags.bits())?;
        self.write_u16_le(0)?; // padding
        Ok(())
    }
}

impl<W: Write + ?Sized> WriteHeader for W {}
