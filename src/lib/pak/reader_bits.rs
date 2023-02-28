use std::{
    fs,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
    path::PathBuf,
};

use encoding_rs::WINDOWS_1252;

use crate::{utils::buffer_to_zstring, PathBufUtils, ReadEx};

pub struct Header {
    pub index_offset: u32,
    pub index_size: u32,
}

impl Header {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        Ok(Self {
            index_offset: r.read_u32_le()?,
            index_size: r.read_u32_le()?,
        })
    }
}

pub struct File {
    pub name: PathBuf,
    pub offset: u32,
    pub size: u32,
}

impl File {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        let mut name_buf = vec![0u8; 56];
        r.read_exact(&mut name_buf)?;

        let name = buffer_to_zstring(&name_buf, WINDOWS_1252)?.to_string();

        Ok(Self {
            name: PathBuf::from_unix(&name),
            offset: r.read_u32_le()?,
            size: r.read_u32_le()?,
        })
    }
}

pub fn read_file_index(
    r: &mut BufReader<fs::File>,
    index_offset: u32,
    file_count: usize,
) -> Result<Vec<File>> {
    let mut files = Vec::with_capacity(file_count);

    r.seek(SeekFrom::Start(u64::from(index_offset)))?;

    for _ in 0..file_count {
        let file = File::read(r)?;
        files.push(file);
    }

    Ok(files)
}
