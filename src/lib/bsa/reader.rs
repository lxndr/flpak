use std::{
    fs,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use libflate::zlib;

use crate::{FileType, ReadEx};

use super::{
    read_file_index::{File, Folder},
    Flags, Hash, ReadFileIndex, ReadHeader, Version, BSA_SIGNATURE,
};

pub struct Reader {
    file: fs::File,
    folders: Vec<Folder>,
    files: Vec<File>,
    version: Version,
    xmem_codec: bool,
}

impl Reader {
    fn open(path: &Path, options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut rdr = BufReader::new(file);

        let signature = rdr
            .read_u8_vec(4)
            .map_err(crate::reader::Error::ReadingSignature)?;

        if signature != BSA_SIGNATURE {
            return Err(crate::reader::Error::InvalidSignature {
                signature,
                expected_signature: BSA_SIGNATURE.to_vec(),
            });
        }

        let hdr = rdr
            .read_header()
            .map_err(crate::reader::Error::ReadingHeader)?;

        // folder records
        let (folders, files) = rdr
            .read_file_index(&hdr)
            .map_err(crate::reader::Error::ReadingFileIndex)?;

        if options.strict {
            for folder in &folders {
                let expected_hash = Hash::from_folder_path(&folder.name);

                if folder.name_hash != expected_hash {
                    return Err(crate::reader::Error::InvalidFileNameHash {
                        filename: folder.name.clone(),
                        hash: format!("{:016x}", folder.name_hash),
                        expected_hash: format!("{expected_hash:016x}"),
                    });
                }
            }

            for file in &files {
                let filename_pos = file.name.rfind('/');
                let filename = match filename_pos {
                    Some(pos) => &file.name[pos + 1..],
                    None => &file.name,
                };

                let expected_hash = Hash::from_file_name(filename);

                if file.name_hash != expected_hash {
                    return Err(crate::reader::Error::InvalidFileNameHash {
                        filename: file.name.clone(),
                        hash: format!("{:016x}", file.name_hash),
                        expected_hash: format!("{expected_hash:016x}"),
                    });
                }
            }
        }

        Ok(Reader {
            file: rdr.into_inner(),
            folders,
            files,
            version: hdr.version,
            xmem_codec: hdr.flags.contains(Flags::XMEM_CODEC),
        })
    }
}

impl crate::reader::Reader for Reader {
    fn file_count(&self) -> usize {
        self.folders.len() + self.files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        let folder_count = self.folders.len();

        if index < folder_count {
            let folder = self
                .folders
                .get(index)
                .expect("`index` should be within boundaries");

            crate::reader::File {
                name: folder.name.clone(),
                file_type: FileType::Directory,
                size: None,
            }
        } else {
            let file = self
                .files
                .get(index - folder_count)
                .expect("`index` should be within boundaries");

            crate::reader::File {
                name: file.name.clone(),
                file_type: FileType::RegularFile,
                size: Some(u64::from(file.size)),
            }
        }
    }

    fn create_file_reader<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn Read + 'a>> {
        let folder_count = self.folders.len();

        if index < folder_count {
            return Err(crate::reader::Error::NotFile);
        }

        let file_rec = self
            .files
            .get(index - folder_count)
            .expect("`index` should be within boundaries");

        self.file
            .seek(SeekFrom::Start(u64::from(file_rec.offset)))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let data_stm = self.file.by_ref().take(u64::from(file_rec.size));

        if file_rec.compressed {
            if self.xmem_codec {
                return Err(crate::reader::Error::Unsupported(
                    "xmem compression are not supported".into(),
                ));
            }

            match self.version {
                Version::V103 | Version::V104 => {
                    let decoder = zlib::Decoder::new(data_stm)
                        .map_err(crate::reader::Error::ReadingInputFile)?;
                    return Ok(Box::new(decoder));
                }
                Version::V105 => {
                    let decoder = lz4_flex::frame::FrameDecoder::new(data_stm);
                    return Ok(Box::new(decoder));
                }
            }
        }

        Ok(Box::new(data_stm))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
