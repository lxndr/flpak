use std::{
    fs,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result, Seek, SeekFrom},
};

use crate::ReadEx;

pub struct Header {
    pub version: u32,
    pub folder_records_offset: u32,
    pub archive_flags: u32,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: u16,
}

impl Header {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        Ok(Self {
            version: r.read_u32_le()?,
            folder_records_offset: r.read_u32_le()?,
            archive_flags: r.read_u32_le()?,
            folder_count: r.read_u32_le()?,
            file_count: r.read_u32_le()?,
            total_folder_name_length: r.read_u32_le()?,
            total_file_name_length: r.read_u32_le()?,
            file_flags: r.read_u16_le()?,
        })
    }
}

pub struct Folder {
    pub name: String,
    pub name_hash: u64,
    pub file_count: u32,
    pub offset: u32,
}

pub fn read_folder_records(
    r: &mut BufReader<fs::File>,
    folder_count: u32,
    version: u32,
    big_endian: bool,
) -> Result<Vec<Folder>> {
    let mut folders = Vec::new();

    for _ in 0..folder_count {
        let name_hash = r.read_u64(big_endian)?;
        let file_count = r.read_u32(big_endian)?;

        if version == 105 {
            r.seek_relative(4)?; // padding
        }

        let offset = r.read_u32(big_endian)?;

        if version == 105 {
            r.seek_relative(4)?; // padding
        }

        folders.push(Folder {
            name: String::new(),
            name_hash,
            file_count,
            offset,
        });
    }

    Ok(folders)
}

pub struct File {
    pub name: String,
    pub name_hash: u64,
    pub size: u32,
    pub original_size: u32,
    pub offset: u32,
    pub compressed: bool,
}

pub fn read_file_records(
    r: &mut BufReader<fs::File>,
    folders: &mut Vec<Folder>,
    has_folder_names: bool,
    compressed_by_default: bool,
    big_endian: bool,
    total_file_name_length: u32,
) -> Result<Vec<File>> {
    let mut files = Vec::new();

    for folder in folders.iter_mut() {
        r.seek(SeekFrom::Start(
            u64::from(folder.offset) - u64::from(total_file_name_length),
        ))?;

        if has_folder_names {
            let name = read_bzstring(r)?;
            folder.name = name.replace('\\', "/");
        }

        for _ in 0..folder.file_count {
            let name_hash = r.read_u64(big_endian)?;
            let mut size = r.read_u32(big_endian)?;
            let offset = r.read_u32(big_endian)?;
            let mut compressed = compressed_by_default;

            if size & 0x40000000 != 0 {
                compressed = !compressed;
                size ^= 0x40000000;
            }

            files.push(File {
                name: folder.name.clone(),
                name_hash,
                size,
                original_size: size,
                offset,
                compressed,
            });
        }
    }

    Ok(files)
}

pub fn read_file_names(
    r: &mut BufReader<fs::File>,
    files: &mut Vec<File>,
    has_file_names: bool,
    total_file_name_length: u32,
) -> Result<()> {
    if has_file_names {
        let mut buf = vec![0u8; total_file_name_length as usize];
        r.read_exact(&mut buf)?;
        let str_data = String::from_utf8_lossy(&buf);
        let file_names: Vec<&str> = str_data.split_terminator('\0').collect();

        for (file, file_name) in files.iter_mut().zip(file_names) {
            file.name.push('/');
            file.name.push_str(file_name);
        }
    }

    Ok(())
}

pub fn read_file_blocks(
    r: &mut BufReader<fs::File>,
    files: &mut Vec<File>,
    embed_file_names: bool,
    big_endian: bool,
) -> Result<()> {
    for file in files {
        r.seek(SeekFrom::Start(u64::from(file.offset)))?;

        if embed_file_names {
            let name = read_bstring(r)?;
            file.name = name.replace('\\', "/");
        }

        if file.compressed {
            file.original_size = r.read_u32(big_endian)?;
        }
    }

    Ok(())
}

pub fn read_bzstring(r: &mut impl BufRead) -> Result<String> {
    let len = r.read_u8()? as usize;

    if len == 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "String cannot be 0 length",
        ));
    }

    let mut string_buf = vec![0u8; len];
    r.read_exact(&mut string_buf)?;
    Ok(String::from_utf8_lossy(&string_buf[..len - 1]).to_string())
}

pub fn read_bstring(r: &mut impl BufRead) -> Result<String> {
    let len = r.read_u8()? as usize;

    if len == 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "String cannot be 0 length",
        ));
    }

    let mut string_buf = vec![0u8; len];
    r.read_exact(&mut string_buf)?;
    Ok(String::from_utf8_lossy(&string_buf[..len]).to_string())
}
