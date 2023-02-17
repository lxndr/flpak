use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::{fs, io};

use super::common::{LocalFileHeader, StructDeserializer, LOCAL_FILE_HEADER_SIGNATURE, END_OF_CENTRAL_DIRECTORY_SIGNATURE, ZIP64_END_OF_CENTRAL_DIRECTORY_SIGNATURE};

use crate::FileType;

struct File {
    name: String,
    file_type: FileType,
    size: u64,
    offset: u64,
}

pub struct Reader {
    stm: BufReader<fs::File>,
    files: Vec<File>,
}

impl Reader {
    fn open(path: &Path, _options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let metadata = file
            .metadata()
            .map_err(crate::reader::Error::ReadingInputFileMetadata)?;
        let mut stm = BufReader::new(file);

        Self::seek_end_of_central_directory(&mut stm)?;

        let end_of_central_directory = EndOfCentralDirectory

        let mut files = Vec::new();

        loop {
            let cur_pos = stm
                .stream_position()
                .map_err(crate::reader::Error::ReadingInputFile)?;

            if cur_pos >= metadata.len() {
                break;
            }

            let mut signature = [0u8; 4];
            stm.read_exact(&mut signature)
                .map_err(crate::reader::Error::ReadingSignature)?;

            match signature {
                LOCAL_FILE_HEADER_SIGNATURE => {
                    let hdr = LocalFileHeader::deserialize(&mut stm)
                        .map_err(crate::reader::Error::ReadingHeader)?;

                    let offset = stm
                        .stream_position()
                        .map_err(crate::reader::Error::ReadingInputFile)?;

                    files.push(File {
                        name: hdr.filename,
                        file_type: FileType::RegularFile,
                        size: hdr.uncompressed_size.into(),
                        offset,
                    });

                    stm.seek_relative(hdr.compressed_size.into())
                        .map_err(crate::reader::Error::ReadingInputFile)?;
                }
                CENTRAL_DIRECTORY_SIGNATURE => {
                    let hdr = LocalFileHeader::deserialize(&mut stm)
                        .map_err(crate::reader::Error::ReadingHeader)?;

                }
                _ => {
                    return Err(crate::reader::Error::InvalidSignature {
                        signature,
                        expected_signature: &LOCAL_FILE_HEADER_SIGNATURE,
                    });
                }
            }
        }

        Ok(Reader { stm, files })
    }

    fn (r: &mut BufReader<fs::File>) -> crate::reader::Result<()> {
        let mut sig = [0u8; 4];
        let offset: i64 = -22;

        loop {
            r.seek(SeekFrom::End(offset))
                .map_err(crate::reader::Error::ReadingInputFile)?;
            r.read_exact(&mut sig)
                .map_err(crate::reader::Error::ReadingInputFile)?;

            match sig {
                END_OF_CENTRAL_DIRECTORY_SIGNATURE => {

                }
                ZIP64_END_OF_CENTRAL_DIRECTORY_SIGNATURE => {

                }
            }
        }
    }
}

impl crate::reader::Reader for Reader {
    fn len(&self) -> usize {
        self.files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        let file = self
            .files
            .get(index)
            .expect("`index` argument should be valid");

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
        let file = self
            .files
            .get(index)
            .expect("`index` argument should be valid");

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
