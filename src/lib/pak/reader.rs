use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::{fs, io};

use super::reader_bits::read_file_index;
use super::{
    common::PAK_SIGNATURE,
    reader_bits::{File, Header},
};

use crate::{FileType, ReadEx};

pub struct Reader {
    stm: BufReader<fs::File>,
    files: Vec<File>,
}

impl Reader {
    fn open(path: &Path, _options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut stm = BufReader::new(file);

        let signature = stm
            .read_u8_vec(4)
            .map_err(crate::reader::Error::ReadingSignature)?;

        if !signature.eq(PAK_SIGNATURE) {
            return Err(crate::reader::Error::InvalidSignature {
                signature,
                expected_signature: PAK_SIGNATURE.to_vec(),
            });
        }

        // header
        let hdr = Header::read(&mut stm).map_err(crate::reader::Error::ReadingHeader)?;

        if hdr.index_size % 64 > 0 {
            return Err(crate::reader::Error::InvalidHeader(
                "file index size should be a multiple of 64".to_string(),
            ));
        }

        let file_count = usize::try_from(hdr.index_size / 64)
            .expect("usize should be large enough to hold file count");

        // files
        let mut files = read_file_index(&mut stm, hdr.index_offset, file_count)
            .map_err(crate::reader::Error::ReadingFileIndex)?;

        // sort by offset
        files.sort_by_key(|f| f.offset);

        Ok(Reader { stm, files })
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
            size: Some(u64::from(file.size)),
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

        self.stm
            .seek(SeekFrom::Start(u64::from(file.offset)))
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
