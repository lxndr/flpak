use std::{
    io::{BufRead, Result, Seek, SeekFrom},
    path::PathBuf,
};

use encoding_rs::WINDOWS_1252;

use crate::{io_error, PathBufUtils, ReadEx};

use super::{hash::ReadHash, Flags, Hash, Header, Version};

pub struct Folder {
    pub name: PathBuf,
    pub name_hash: Hash,
    pub file_count: u32,
    pub offset: u32,
}

pub struct File {
    pub name: PathBuf,
    pub name_hash: Hash,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub compressed: bool,
    pub offset: u32,
    pub data_offset: u32,
}

pub trait ReadFileIndex: BufRead + Seek {
    #[inline]
    fn read_folder_record(&mut self, hdr: &Header) -> Result<Folder> {
        let is_xbox = hdr.flags.contains(Flags::XBOX);
        let name_hash = self.read_hash(is_xbox)?;
        let file_count = self.read_u32_le()?;

        if hdr.version == Version::V105 {
            self.read_u32_le()?; // padding
        }

        let offset = self.read_u32_le()?;

        if hdr.version == Version::V105 {
            self.read_u32_le()?; // padding
        }

        Ok(Folder {
            name: PathBuf::new(),
            name_hash,
            file_count,
            offset,
        })
    }

    #[inline]
    fn read_file_index(&mut self, hdr: &Header) -> Result<(Vec<Folder>, Vec<File>)> {
        let has_folder_names = hdr.flags.contains(Flags::HAS_FOLDER_NAMES);
        let has_file_names = hdr.flags.contains(Flags::HAS_FILE_NAMES);
        let compressed_by_default = hdr.flags.contains(Flags::COMPRESSED_BY_DEFAULT);
        let is_xbox = hdr.flags.contains(Flags::XBOX);
        let embed_file_names = hdr.embedded_file_names();

        let mut folders = Vec::new();
        let mut files = Vec::new();

        // folder records
        self.seek(SeekFrom::Start(hdr.folder_records_offset.into()))?;

        for _ in 0..hdr.folder_count {
            folders.push(self.read_folder_record(hdr)?);
        }

        // file records
        for folder in &mut folders {
            let folder_offset = u64::from(folder.offset) - u64::from(hdr.total_file_name_length);
            self.seek(SeekFrom::Start(folder_offset))?;

            if has_folder_names {
                let path = self.read_u8_zstring(WINDOWS_1252)?;
                folder.name = PathBuf::from_win(&path);
            }

            for _ in 0..folder.file_count {
                let name_hash = self.read_hash(is_xbox)?;
                let mut size = self.read_u32_le()?;
                let offset = self.read_u32_le()?;
                let mut compressed = compressed_by_default;

                if size & 0x40000000 != 0 {
                    compressed = !compressed;
                    size &= 0x3FFFFFFF;
                }

                files.push(File {
                    name: folder.name.clone(), // the file name will be appended later
                    name_hash,
                    packed_size: size,
                    unpacked_size: size,
                    compressed,
                    offset,
                    data_offset: offset,
                });
            }
        }

        // file names
        if has_file_names {
            let buf_len = hdr
                .total_file_name_length
                .try_into()
                .expect("should fit into `usize`");

            self.read_u8_vec(buf_len)?
                .split(|&ch| ch == 0)
                .map(|bytes| {
                    let (cow, _, _) = WINDOWS_1252.decode(bytes);
                    cow.to_string()
                })
                .zip(&mut files)
                .for_each(|(file_name, file)| {
                    file.name.push(file_name);
                });
        }

        // file data blocks
        for file in &mut files {
            self.seek(SeekFrom::Start(file.offset.into()))?;

            if embed_file_names {
                let full_path = self.read_u8_string(WINDOWS_1252).map_err(|err| {
                    io_error!(InvalidData, "failed to read embedded file name: {err}")
                })?;

                let full_path = PathBuf::from_win(&full_path);

                if has_folder_names && has_folder_names {
                    if full_path != file.name {
                        return Err(io_error!(
                            InvalidData,
                            "invalid file name `{}` found in data block",
                            full_path.display(),
                        ));
                    }
                } else {
                    file.name = full_path;
                }
            }

            if file.compressed {
                file.unpacked_size = self.read_u32_le()?;
            }

            // real data offset
            file.data_offset = self
                .stream_position()?
                .try_into()
                .expect("should fit into `u32`");
            file.packed_size -= file.data_offset - file.offset;
        }

        files.sort_by_key(|file| file.data_offset);

        Ok((folders, files))
    }
}

impl<R: BufRead + Seek + ?Sized> ReadFileIndex for R {}
