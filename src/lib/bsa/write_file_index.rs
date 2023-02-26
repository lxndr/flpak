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
        let file_count: u32 = folder
            .files
            .len()
            .try_into()
            .expect("should fit into `u32`");
        let folder_offset = folder.offset + hdr.total_file_name_length;

        self.write_hash(&folder.name_hash)?;
        self.write_u32_le(file_count)?;

        if hdr.version == Version::V105 {
            self.write_u32_le(0)?; // padding
        }

        self.write_u32_le(folder_offset)?;

        if hdr.version == Version::V105 {
            self.write_u32_le(0)?; // padding
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

        // folder records
        for folder in folders {
            self.write_folder_record(folder, hdr)?;
        }

        // file records
        for folder in folders {
            if has_folder_names {
                self.write_u8_zstring(&folder.name)?;
            }

            for file in &folder.files {
                self.write_hash(&file.name_hash)?;
                self.write_u32_le(file.size)?;
                self.write_u32_le(file.offset)?;
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
