use std::{
    io::{Result, Write},
    path::PathBuf,
};

use crate::WriteEx;

use super::{Flags, Hash, Header, Version, WriteHash};

pub struct Folder {
    pub name_hash: Hash,
    pub name: String,
    pub offset: u32,
    pub files: Vec<File>,
}

pub struct File {
    pub local_path: PathBuf,
    pub name: String,
    pub size: u32,
    pub offset: u32,
    pub name_hash: Hash,
}

pub trait WriteFileIndex: Write {
    #[inline]
    fn write_folder_record(&mut self, folder: &Folder, hdr: &Header) -> Result<()> {
        let big_endian = hdr.flags.contains(Flags::BIG_ENDIAN);
        let file_count: u32 = folder
            .files
            .len()
            .try_into()
            .expect("should fit into `u32`");
        let folder_offset = folder.offset + hdr.total_file_name_length;

        self.write_hash(&folder.name_hash, big_endian)?;
        self.write_u32(file_count, big_endian)?;

        if hdr.version == Version::V105 {
            self.write_u32(0, big_endian)?; // padding
        }

        self.write_u32(folder_offset, big_endian)?;

        if hdr.version == Version::V105 {
            self.write_u32(0, big_endian)?; // padding
        }

        Ok(())
    }

    #[inline]
    fn write_file_index(
        &mut self,
        folders: &Vec<Folder>,
        names: &String,
        hdr: &Header,
    ) -> Result<()> {
        let has_folder_names = hdr.flags.contains(Flags::HAS_FOLDER_NAMES);
        let has_file_names = hdr.flags.contains(Flags::HAS_FILE_NAMES);
        let big_endian = hdr.flags.contains(Flags::BIG_ENDIAN);

        // folder records
        for folder in folders {
            self.write_folder_record(folder, &hdr)?;
        }

        // file records
        for folder in folders {
            if has_folder_names {
                self.write_u8_zstring(&folder.name)?;
            }

            for file in &folder.files {
                self.write_hash(&file.name_hash, big_endian)?;
                self.write_u32(file.size, big_endian)?;
                self.write_u32(file.offset, big_endian)?;
            }
        }

        // file names
        if has_file_names {
            self.write_all(names.as_bytes())?;
        }

        Ok(())
    }
}

impl<R: Write + ?Sized> WriteFileIndex for R {}
