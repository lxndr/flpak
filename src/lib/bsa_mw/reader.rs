use std::{
    fs, io,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use crate::FileType;

use super::{
    hash::Hash,
    records::{read_file_index, read_file_names, Header, BSA_SIGNATURE},
};

struct FileEntry {
    name: String,
    size: u32,
    offset: u32,
}

pub struct Reader {
    file: fs::File,
    files: Vec<FileEntry>,
    data_offset: u64,
}

impl Reader {
    fn open(path: &Path, options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut rdr = io::BufReader::new(file);

        // header
        let hdr = Header::read(&mut rdr).map_err(crate::reader::Error::ReadingHeader)?;

        if hdr.signature != BSA_SIGNATURE {
            return Err(crate::reader::Error::InvalidSignature {
                signature: hdr.signature.to_vec(),
                expected_signature: BSA_SIGNATURE.to_vec(),
            });
        }

        let hash_table_offset = hdr.absolute_hash_table_offset();
        let file_count = usize::try_from(hdr.file_count).expect("should fit into `usize`");

        // file records
        let file_index = read_file_index(&mut rdr, file_count)
            .map_err(crate::reader::Error::ReadingFileIndex)?;

        // file names
        let names = read_file_names(&mut rdr, &hdr, file_count)
            .map_err(crate::reader::Error::ReadingFileName)?;

        // hashes
        let rdr_pos = rdr
            .stream_position()
            .map_err(crate::reader::Error::ReadingInputFile)?;

        if hash_table_offset < rdr_pos {
            return Err(crate::reader::Error::Other("hash table offset {hash_table_offset:010x} is incorrect, expected to be equal or greater than {stm_pos:010x}".into()));
        }

        rdr.seek(SeekFrom::Start(hash_table_offset))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        if options.strict {
            for name in &names {
                let hash =
                    Hash::read_from(&mut rdr).map_err(crate::reader::Error::ReadingInputFile)?;
                let expected_hash = Hash::from_path(&name);

                if hash != expected_hash {
                    return Err(crate::reader::Error::InvalidFileNameHash {
                        filename: name.clone(),
                        hash: hash.to_string(),
                        expected_hash: expected_hash.to_string(),
                    });
                }
            }
        } else {
            rdr.seek_relative((file_count * 8) as i64)
                .map_err(crate::reader::Error::ReadingInputFile)?;
        }

        // files
        let mut files: Vec<FileEntry> = file_index
            .iter()
            .zip(&names)
            .map(|(rec, name)| FileEntry {
                name: name.clone(),
                size: rec.size,
                offset: rec.offset,
            })
            .collect();

        // sort by offset for easier continuos extraction
        files.sort_unstable_by_key(|f| f.offset);

        // data offset
        let data_offset = rdr
            .stream_position()
            .map_err(crate::reader::Error::ReadingInputFile)?;

        Ok(Reader {
            file: rdr.into_inner(),
            files,
            data_offset,
        })
    }
}

impl crate::reader::Reader for Reader {
    fn file_count(&self) -> usize {
        self.files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        let file = self
            .files
            .get(index)
            .expect("`index` should be within boundaries");

        crate::reader::File {
            name: file.name.clone(),
            file_type: FileType::RegularFile,
            size: Some(file.size.into()),
        }
    }

    fn create_file_reader<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn io::Read + 'a>> {
        let file = self
            .files
            .get(index)
            .expect("`index` should be within boundaries");
        self.file
            .seek(SeekFrom::Start(self.data_offset + u64::from(file.offset)))
            .map_err(crate::reader::Error::ReadingInputFile)?;
        let rdr = self.file.by_ref().take(file.size.into());
        Ok(Box::new(rdr))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
