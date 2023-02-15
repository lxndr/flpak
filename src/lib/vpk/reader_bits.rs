use std::{
    io::{BufRead, Error, ErrorKind, Result},
    str,
};

use crate::ReadEx;

pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub file_tree_size: u32,
    pub file_data_section_size: u32,
    pub archive_md5_section_size: u32,
    pub other_md5_section_size: u32,
    pub signature_section_size: u32,
}

impl Header {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        let mut signature = [0u8; 4];
        r.read_exact(&mut signature)?;

        let mut hdr = Self {
            signature,
            version: r.read_u32_le()?,
            file_tree_size: r.read_u32_le()?,
            file_data_section_size: 0,
            archive_md5_section_size: 0,
            other_md5_section_size: 0,
            signature_section_size: 0,
        };

        if hdr.version >= 2 {
            hdr.file_data_section_size = r.read_u32_le()?;
            hdr.archive_md5_section_size = r.read_u32_le()?;
            hdr.other_md5_section_size = r.read_u32_le()?;
            hdr.signature_section_size = r.read_u32_le()?;
        }

        Ok(hdr)
    }
}

pub struct File {
    pub name: String,
    pub crc: u32,
    pub preload_bytes: Vec<u8>,
    pub archive_index: Option<u16>,
    pub entry_offset: u32,
    pub entry_length: u32,
}

pub fn read_file_tree(r: &mut impl BufRead) -> Result<Vec<File>> {
    let mut files = Vec::new();

    loop {
        let ext = r.read_zstring()?;

        if ext.is_empty() {
            break;
        };

        loop {
            let basepath = r.read_zstring()?;

            if basepath.is_empty() {
                break;
            };

            loop {
                let filename = r.read_zstring()?;

                if filename.is_empty() {
                    break;
                };

                let crc = r.read_u32_le()?;
                let preload_len = usize::try_from(r.read_u16_le()?).unwrap();
                let archive_index = r.read_u16_le()?;
                let entry_offset = r.read_u32_le()?;
                let entry_length = r.read_u32_le()?;
                let terminator = r.read_u16_le()?;

                if terminator != 0xffff {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "invalid file entry terminator",
                    ));
                }

                // read preload bytes
                let mut preload_bytes = vec![0; preload_len];
                r.read_exact(&mut preload_bytes)?;

                files.push({
                    File {
                        name: format_filepath(&basepath, &filename, &ext),
                        crc,
                        preload_bytes,
                        archive_index: if archive_index == 0x7fff {
                            None
                        } else {
                            Some(archive_index)
                        },
                        entry_offset,
                        entry_length,
                    }
                });
            }
        }
    }

    Ok(files)
}

fn format_filepath(basepath: &str, filename: &str, ext: &str) -> String {
    let mut path = String::new();

    if !basepath.is_empty() {
        path.push_str(basepath);
    }

    path.push_str(filename);
    path.push('.');
    path.push_str(ext);

    path
}
