use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::{fs, io};

use super::{
    common::{BSA_HEADER_SIZE, BSA_SIGNATURE},
    hash::Hash,
    reader_bits::{read_names, FileRecord, Header},
};

use crate::FileType;

struct File {
    name: String,
    size: u32,
    offset: u32,
}

pub struct Reader {
    stm: BufReader<fs::File>,
    files: Vec<File>,
    data_offset: u64,
}

impl Reader {
    fn open(path: &Path, options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let file_metadata = file
            .metadata()
            .map_err(crate::reader::Error::ReadingInputFileMetadata)?;

        if file_metadata.len() > u64::from(u32::MAX) {
            return Err(crate::reader::Error::Other(
                "file cannot be larger than 4GiB",
            ));
        }

        let mut stm = BufReader::new(file);

        // header
        let hdr = Header::read(&mut stm)
            .map_err(|err| crate::reader::Error::Io("failed to read file's header", err))?;

        if !hdr.signature.eq(BSA_SIGNATURE) {
            return Err(crate::reader::Error::InvalidSignature {
                signature: hdr.signature,
                expected_signature: BSA_SIGNATURE,
            });
        }

        let hash_table_offset = u64::from(hdr.hash_table_offset + BSA_HEADER_SIZE);
        let file_count = usize::try_from(hdr.file_count).unwrap();

        // files
        let mut files = Vec::with_capacity(file_count);

        for _ in 0..file_count {
            let FileRecord { size, offset } = FileRecord::read(&mut stm)
                .map_err(|err| crate::reader::Error::Io("failed to read file record", err))?;

            files.push(File {
                name: String::new(),
                size,
                offset,
            });
        }

        // names
        let names = read_names(&mut stm, file_count, hash_table_offset)
            .map_err(|err| crate::reader::Error::Io("failed to read file names", err))?;

        for (file, name) in files.iter_mut().zip(names) {
            file.name = name;
        }

        // hashes
        let stm_pos = stm
            .stream_position()
            .map_err(crate::reader::Error::ReadingInputFile)?;

        if hash_table_offset < stm_pos {
            return Err(crate::reader::Error::Other("hash table offset {hash_table_offset:010x} is incorrect, expected to be equal or greater than {stm_pos:010x}"));
        }

        stm.seek(SeekFrom::Start(hash_table_offset))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        if options.strict {
            for file in &files {
                let hash =
                    Hash::read_from(&mut stm).map_err(crate::reader::Error::ReadingInputFile)?;
                let expected_hash = Hash::from_path(&file.name);

                if hash != expected_hash {
                    return Err(crate::reader::Error::InvalidFileNameHash {
                        filename: file.name.clone(),
                        hash: hash.to_string(),
                        expected_hash: expected_hash.to_string(),
                    });
                }
            }
        } else {
            stm.seek_relative((file_count * 8) as i64)
                .map_err(crate::reader::Error::ReadingInputFile)?;
        }

        // sort by offset
        files.sort_by_key(|f| f.offset);

        // data offset
        let data_offset = stm
            .stream_position()
            .map_err(crate::reader::Error::ReadingInputFile)?;

        Ok(Reader {
            stm,
            files,
            data_offset,
        })
    }
}

impl crate::reader::Reader for Reader {
    fn len(&self) -> usize {
        self.files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        let file = self.files.get(index).unwrap();

        crate::reader::File {
            name: file.name.clone(),
            file_type: FileType::RegularFile,
            size: Some(u64::from(file.size)),
        }
    }

    fn open_file_by_index<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn io::Read + 'a>> {
        let file = self.files.get(index).unwrap();

        self.stm
            .seek(SeekFrom::Start(self.data_offset + u64::from(file.offset)))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let stm = self.stm.by_ref().take(u64::from(file.size));
        Ok(Box::new(stm))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
