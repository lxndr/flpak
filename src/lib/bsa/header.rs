use std::io::{BufRead, Error, ErrorKind, Result, Write};

use bitflags::bitflags;

use crate::{ReadEx, WriteEx};

use super::Version;

pub const BSA_SIGNATURE: &[u8; 4] = b"BSA\0";

bitflags! {
    pub struct Flags: u32 {
        const HAS_FOLDER_NAMES = 1 << 0;
        const HAS_FILE_NAMES   = 1 << 1;
        const COMPRESSED_BY_DEFAULT   = 1 << 2;
        const RETAIN_FOLDER_NAMES   = 1 << 3;
        const RETAIN_FILE_NAMES   = 1 << 4;
        const RETAIN_FILE_NAME_OFFSETS   = 1 << 5;
        const BIG_ENDIAN   = 1 << 6;
        const RETAIN_STRINGS   = 1 << 7;
        const EMBEDDED_FILE_NAMES   = 1 << 8;
        const XMEM_CODEC   = 1 << 9;
    }

    #[derive(Default)]
    pub struct FileFlags: u16 {
        const MESHES = 1 << 0;
        const TEXTURES   = 1 << 1;
        const MENUS   = 1 << 2;
        const SOUNDS   = 1 << 3;
        const VOICES   = 1 << 4;
        const SHADERS   = 1 << 5;
        const TREES   = 1 << 6;
        const FONTS   = 1 << 7;
        const MISC   = 1 << 8;
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
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))?,
            folder_records_offset: self.read_u32_le()?,
            flags: Flags::from_bits(self.read_u32_le()?)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid flags"))?,
            folder_count: self.read_u32_le()?,
            file_count: self.read_u32_le()?,
            total_folder_name_length: self.read_u32_le()?,
            total_file_name_length: self.read_u32_le()?,
            file_flags: FileFlags::from_bits(self.read_u16_le()?)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid file flags"))?,
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
