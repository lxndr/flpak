use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result, Seek},
};

use crate::{utils::buffer_to_ascii_zstring, ReadEx};

pub struct Header {
    pub signature: [u8; 4],
    pub hash_table_offset: u32,
    pub file_count: u32,
}

impl Header {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        let mut signature = [0u8; 4];
        r.read_exact(&mut signature)?;

        let hash_table_offset = r.read_u32_le()?;
        let file_count = r.read_u32_le()?;

        Ok(Self {
            signature,
            hash_table_offset,
            file_count,
        })
    }
}

pub struct FileRecord {
    pub size: u32,
    pub offset: u32,
}

impl FileRecord {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        Ok(Self {
            size: r.read_u32_le()?,
            offset: r.read_u32_le()?,
        })
    }
}

#[inline]
pub fn read_names(
    r: &mut BufReader<File>,
    file_count: usize,
    hash_table_offset: u64,
) -> Result<Vec<String>> {
    let mut name_offsets = Vec::with_capacity(file_count);

    for _ in 0..file_count {
        let offset = r.read_u32_le()?;
        name_offsets.push(offset);
    }

    let names_len = hash_table_offset - r.stream_position()?;
    let mut names_buf = vec![0; usize::try_from(names_len).unwrap()];
    r.read_exact(&mut names_buf)?;

    let mut names = Vec::new();

    for offset in name_offsets {
        let name = buffer_to_ascii_zstring(&names_buf[offset as usize..]).map_err(|err| {
            Error::new(ErrorKind::InvalidData, format!("invalid file name: {err}"))
        })?;

        names.push(name.replace('\\', "/"));
    }

    Ok(names)
}
