use std::{
    fs,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use super::reader_bits::{read_file_index, File, Header};
use crate::FileType;

pub struct Reader {
    stm: BufReader<fs::File>,
    files: Vec<File>,
}

impl Reader {
    fn open(path: &Path, _options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut stm = BufReader::new(file);

        let hdr = Header::read(&mut stm).map_err(crate::reader::Error::ReadingHeader)?;

        if hdr.signature != "RPA-3.0" {
            return Err(crate::reader::Error::InvalidStringSignature {
                signature: hdr.signature,
                expected_signature: "RPA-3.0",
            });
        }

        stm.seek(SeekFrom::Start(hdr.index_offset))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let files =
            read_file_index(&mut stm, hdr.key).map_err(crate::reader::Error::ReadingFileIndex)?;

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
            size: Some(file.size),
        }
    }

    fn create_file_reader<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn Read + 'a>> {
        let file = self
            .files
            .get(index)
            .expect("`index` should be within boundaries");

        self.stm
            .seek(SeekFrom::Start(file.offset))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let stm = self.stm.by_ref().take(file.size);
        Ok(Box::new(stm))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
