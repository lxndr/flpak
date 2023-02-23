use std::io::{BufRead, Error, ErrorKind, Result, Seek, SeekFrom};

use crate::ReadEx;

use super::{Flags, Hash, Header, Version};

pub struct Folder {
    pub name: String,
    pub name_hash: Hash,
    pub file_count: u32,
    pub offset: u32,
}

pub struct File {
    pub name: String,
    pub name_hash: Hash,
    pub size: u32,
    pub original_size: u32,
    pub offset: u32,
    pub compressed: bool,
}

pub trait ReadFileIndex: BufRead + Seek {
    #[inline]
    fn read_folder_record(&mut self, hdr: &Header) -> Result<Folder> {
        let big_endian = hdr.flags.contains(Flags::BIG_ENDIAN);
        let name_hash = Hash::from(self.read_u64(big_endian)?);
        let file_count = self.read_u32(big_endian)?;

        if hdr.version == Version::V105 {
            self.read_u32_le()?; // padding
        }

        let offset = self.read_u32(big_endian)?;

        if hdr.version == Version::V105 {
            self.read_u32_le()?; // padding
        }

        Ok(Folder {
            name: String::new(),
            name_hash,
            file_count,
            offset,
        })
    }

    #[inline]
    fn read_file_index(&mut self, hdr: &Header) -> Result<(Vec<Folder>, Vec<File>)> {
        let has_folder_names = hdr.flags.contains(Flags::HAS_FOLDER_NAMES);
        let has_file_names = hdr.flags.contains(Flags::HAS_FILE_NAMES);
        let big_endian = hdr.flags.contains(Flags::BIG_ENDIAN);
        let compressed_by_default = hdr.flags.contains(Flags::COMPRESSED_BY_DEFAULT);
        let embed_file_names = hdr.flags.contains(Flags::EMBEDDED_FILE_NAMES);

        let mut folders = Vec::new();
        let mut files = Vec::new();

        // folder records
        self.seek(SeekFrom::Start(u64::from(hdr.folder_records_offset)))?;

        for _ in 0..hdr.folder_count {
            folders.push(self.read_folder_record(&hdr)?);
        }

        // file records
        for folder in &mut folders {
            let folder_offset = u64::from(folder.offset) - u64::from(hdr.total_file_name_length);
            self.seek(SeekFrom::Start(folder_offset))?;

            if has_folder_names {
                let name = self.read_u8_zstring()?;
                folder.name = name.replace('\\', "/");
            }

            for _ in 0..folder.file_count {
                let name_hash = Hash::from(self.read_u64(big_endian)?);
                let mut size = self.read_u32(big_endian)?;
                let offset = self.read_u32(big_endian)?;
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
            let str_data = String::from_utf8(buf).map_err(|err| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid file names block: {}", err.to_string()),
                )
            })?;
            let file_names: Vec<&str> = str_data.split_terminator('\0').collect();

            if file_names.len() < files.len() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "invalid number of file names found {}, expected at least {}",
                        file_names.len(),
                        files.len()
                    ),
                ));
            }

            for (file, file_name) in files.iter_mut().zip(file_names) {
                file.name.push('/');
                file.name.push_str(file_name);
            }
        }

        // file data blocks
        for file in &mut files {
            self.seek(SeekFrom::Start(u64::from(file.offset)))?;

            if embed_file_names {
                let name = self.read_u8_string()?;
                file.name = name.replace('\\', "/");
            }

            if file.compressed {
                file.original_size = self.read_u32(big_endian)?;
            }
        }

        Ok((folders, files))
    }
}

impl<R: BufRead + Seek + ?Sized> ReadFileIndex for R {}
