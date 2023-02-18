use std::io;

use crate::ReadEx;

pub const BA2_SIGNATURE: &str = "BTDX";

#[repr(C, packed)]
pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub archive_type: [u8; 4],
    pub num_files: u32,
    pub names_offset: u64,
}

impl Header {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Self> {
        let mut hdr: Self = r.read_c_struct()?;

        if cfg!(target_endian = "big") {
            hdr.version = hdr.version.swap_bytes();
            hdr.num_files = hdr.num_files.swap_bytes();
            hdr.names_offset = hdr.names_offset.swap_bytes();
        }

        Ok(hdr)
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct GeneralBlock {
    pub name_hash: u32,
    pub ext: [u8; 4],
    pub dir_hash: u32,
    pub unknown_1: u32, // flags?
    pub offset: u64,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub padding: u32, // always 0xBAADFOOD
}

impl GeneralBlock {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Self> {
        let mut hdr: Self = r.read_c_struct()?;

        if cfg!(target_endian = "big") {
            hdr.name_hash = hdr.name_hash.swap_bytes();
            hdr.dir_hash = hdr.dir_hash.swap_bytes();
            hdr.unknown_1 = hdr.unknown_1.swap_bytes();
            hdr.offset = hdr.offset.swap_bytes();
            hdr.packed_size = hdr.packed_size.swap_bytes();
            hdr.unpacked_size = hdr.unpacked_size.swap_bytes();
            hdr.padding = hdr.padding.swap_bytes();
        }

        Ok(hdr)
    }
}

#[repr(C, packed)]
pub struct TextureBlock {
    pub name_hash: u32,
    pub ext: [u8; 4],
    pub dir_hash: u32,
    pub unknown_1: u8,
    pub num_chunks: u8,
    pub chunk_hdr_size: u16,
    pub height: u16,
    pub width: u16,
    pub num_mips: u8,
    pub format: u8,
    pub unknown_2: u16,
}

impl TextureBlock {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Self> {
        let mut rec: Self = r.read_c_struct()?;

        if cfg!(target_endian = "big") {
            rec.name_hash = rec.name_hash.swap_bytes();
            rec.dir_hash = rec.dir_hash.swap_bytes();
            rec.chunk_hdr_size = rec.chunk_hdr_size.swap_bytes();
            rec.height = rec.height.swap_bytes();
            rec.width = rec.width.swap_bytes();
            rec.unknown_2 = rec.unknown_2.swap_bytes();
        }

        Ok(rec)
    }
}

#[repr(C, packed)]
pub struct TextureChunk {
    pub offset: u64,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub start_mip: u16,
    pub end_mip: u16,
    pub unknown_1: u32,
}

impl TextureChunk {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Self> {
        let mut rec: Self = r.read_c_struct()?;

        if cfg!(target_endian = "big") {
            rec.offset = rec.offset.swap_bytes();
            rec.packed_size = rec.packed_size.swap_bytes();
            rec.unpacked_size = rec.unpacked_size.swap_bytes();
            rec.start_mip = rec.start_mip.swap_bytes();
            rec.end_mip = rec.end_mip.swap_bytes();
            rec.unknown_1 = rec.unknown_1.swap_bytes();
        }

        Ok(rec)
    }
}

pub struct TextureInfo {
    pub hdr: TextureBlock,
    pub chunks: Vec<TextureChunk>,
}
