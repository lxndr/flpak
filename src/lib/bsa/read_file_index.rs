use std::{
    io::{BufRead, Result, Seek, SeekFrom},
    path::PathBuf,
};

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
    pub size: u32,
    pub original_size: u32,
    pub offset: u32,
    pub compressed: bool,
}

pub trait ReadFileIndex: BufRead + Seek {
    #[inline]
    fn read_folder_record(&mut self, hdr: &Header) -> Result<Folder> {
        let name_hash = self.read_hash()?;
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
        let embed_file_names = hdr.embedded_file_names();

        let mut folders = Vec::new();
        let mut files = Vec::new();

        // folder records
        self.seek(SeekFrom::Start(u64::from(hdr.folder_records_offset)))?;

        for _ in 0..hdr.folder_count {
            folders.push(self.read_folder_record(hdr)?);
        }

        // file records
        for folder in &mut folders {
            let folder_offset = u64::from(folder.offset) - u64::from(hdr.total_file_name_length);
            self.seek(SeekFrom::Start(folder_offset))?;

            if has_folder_names {
                let path = self.read_u8_zstring()?;
                folder.name = PathBuf::try_from_ascii_win(&path)
                    .map_err(|err| io_error!(InvalidData, "invalid folder name `{path}`: {err}"))?;
            }

            for _ in 0..folder.file_count {
                let name_hash = self.read_hash()?;
                let mut size = self.read_u32_le()?;
                let offset = self.read_u32_le()?;
                let mut compressed = compressed_by_default;

                if size & 0x40000000 != 0 {
                    compressed = !compressed;
                    size ^= 0x40000000;
                }

                files.push(File {
                    name: folder.name.clone(), // the file name will be appended later
                    name_hash,
                    size,
                    original_size: size,
                    offset,
                    compressed,
                });
            }
        }

        // file names
        if has_file_names {
            let buf_len =
                usize::try_from(hdr.total_file_name_length).expect("should fit into `usize`");
            let buf = self.read_u8_vec(buf_len)?;
            let str_data = String::from_utf8(buf)
                .map_err(|err| io_error!(InvalidData, "invalid file names block: {err}"))?;
            let file_names: Vec<&str> = str_data.split_terminator('\0').collect();

            if file_names.len() < files.len() {
                return Err(io_error!(
                    InvalidData,
                    "invalid number of file names found {}, expected at least {}",
                    file_names.len(),
                    files.len()
                ));
            }

            for (file, file_name) in files.iter_mut().zip(file_names) {
                file.name.push(file_name);
            }
        }

        // file data blocks
        for file in &mut files {
            self.seek(SeekFrom::Start(u64::from(file.offset)))?;

            if embed_file_names {
                let full_path = self.read_u8_string().map_err(|err| {
                    io_error!(InvalidData, "failed to read embedded file name: {err}")
                })?;
                let full_path =
                    PathBuf::try_from_ascii_win(&full_path).map_err(|err| {
                        io_error!(InvalidData, "invalid file name `{full_path}`: {err}")
                    })?;

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
                file.original_size = self.read_u32_le()?;
            }
        }

        Ok((folders, files))
    }
}

impl<R: BufRead + Seek + ?Sized> ReadFileIndex for R {}
