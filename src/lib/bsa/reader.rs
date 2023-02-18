use std::fs;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::FileType;

use super::reader_bits::read_folder_records;
use super::{
    common::BSA_SIGNATURE,
    hash::{calc_file_name_hash, calc_folder_name_hash},
    reader_bits::{read_file_blocks, read_file_names, read_file_records, File, Folder, Header},
};

const VALID_VERSIONS: [u32; 3] = [103, 104, 105];

pub struct Reader {
    stm: BufReader<fs::File>,
    folders: Vec<Folder>,
    files: Vec<File>,
    xmem_codec: bool,
}

impl Reader {
    fn open(path: &Path, options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let file_metadata = file
            .metadata()
            .map_err(crate::reader::Error::ReadingInputFileMetadata)?;
        let mut stm = BufReader::new(file);

        if file_metadata.len() > u64::from(u32::MAX) {
            return Err(crate::reader::Error::Other(
                "file cannot be larger than 4GiB".into(),
            ));
        }

        let mut signature = [0u8; 4];
        stm.read_exact(&mut signature)
            .map_err(|err| crate::reader::Error::Io("failed to read file signature", err))?;

        if !signature.eq(BSA_SIGNATURE) {
            return Err(crate::reader::Error::InvalidSignature {
                signature,
                expected_signature: BSA_SIGNATURE,
            });
        }

        let hdr = Header::read(&mut stm)
            .map_err(|err| crate::reader::Error::Io("failed to read file header", err))?;

        if !VALID_VERSIONS.contains(&hdr.version) {
            return Err(crate::reader::Error::UnsupportedVersion {
                version: hdr.version,
                supported_versions: &VALID_VERSIONS,
            });
        }

        if options.strict && hdr.folder_records_offset != 36 {
            return Err(crate::reader::Error::Other(
                "invalid folder records offset".into(),
            ));
        }

        let has_folder_names = (hdr.archive_flags & 0x01) != 0;
        let has_file_names = (hdr.archive_flags & 0x02) != 0;
        let compressed_by_default = (hdr.archive_flags & 0x04) != 0;
        let is_big_endian = (hdr.archive_flags & 0x40) != 0;
        let embed_file_names = (hdr.archive_flags & 0x100) != 0;
        let xmem_codec = (hdr.archive_flags & 0x200) != 0;

        // folder records
        stm.seek(SeekFrom::Start(u64::from(hdr.folder_records_offset)))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let mut folders =
            read_folder_records(&mut stm, hdr.folder_count, hdr.version, is_big_endian)
                .map_err(|err| crate::reader::Error::Io("failed to read folder records", err))?;

        let mut files = read_file_records(
            &mut stm,
            &mut folders,
            has_folder_names,
            compressed_by_default,
            is_big_endian,
            hdr.total_file_name_length,
        )
        .map_err(|err| crate::reader::Error::Io("failed to read file records", err))?;

        read_file_names(
            &mut stm,
            &mut files,
            has_file_names,
            hdr.total_file_name_length,
        )
        .map_err(|err| crate::reader::Error::Io("failed to read file names", err))?;

        read_file_blocks(&mut stm, &mut files, embed_file_names, is_big_endian)
            .map_err(|err| crate::reader::Error::Io("failed to read file names", err))?;

        if options.strict {
            for folder in &folders {
                let expected_hash = calc_folder_name_hash(&folder.name);

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

                let expected_hash = calc_file_name_hash(filename);

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
            stm,
            folders,
            files,
            xmem_codec,
        })
    }
}

impl crate::reader::Reader for Reader {
    fn len(&self) -> usize {
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

    fn open_file_by_index<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn Read + 'a>> {
        let folder_count = self.folders.len();

        if index < folder_count {
            return Err(crate::reader::Error::NotFile);
        }

        let file = self.files.get(index - folder_count).unwrap();

        self.stm
            .seek(SeekFrom::Start(u64::from(file.offset)))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let data_stm = self.stm.by_ref().take(u64::from(file.size));

        if file.compressed {
            return Err(crate::reader::Error::Unsupported(
                "compressed files are not supported",
            ));
        }

        if self.xmem_codec {
            return Err(crate::reader::Error::Unsupported(
                "xmem compression are not supported",
            ));
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
