use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use crate::FileType;

use super::{
    common::VPK_SIGNATURE,
    reader_bits::{read_file_tree, File, Header},
};

const VALID_VERSIONS: [u32; 2] = [1, 2];

pub struct Reader {
    dir_file: BufReader<fs::File>,
    dat_files: HashMap<u16, BufReader<fs::File>>,
    files: Vec<File>,
    file_data_offset: u64,
}

impl Reader {
    fn open(path: &Path, _options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut stm = BufReader::new(file);

        // header
        let hdr = Header::read(&mut stm)
            .map_err(|err| crate::reader::Error::Io("failed to read file's header", err))?;

        if !hdr.signature.eq(VPK_SIGNATURE) {
            return Err(crate::reader::Error::InvalidSignature {
                signature: hdr.signature,
                expected_signature: VPK_SIGNATURE,
            });
        }

        if !VALID_VERSIONS.contains(&hdr.version) {
            return Err(crate::reader::Error::UnsupportedVersion {
                version: hdr.version,
                supported_versions: &VALID_VERSIONS,
            });
        }

        let header_size = stm
            .stream_position()
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let mut dat_files = HashMap::new();
        let files = read_file_tree(&mut stm)
            .map_err(|err| crate::reader::Error::Io("failed to read file tree", err))?;

        for file in &files {
            if let Some(archive_index) = file.archive_index {
                if dat_files.get(&archive_index).is_none() {
                    let file_name = path.to_str().unwrap();
                    let mut archive_path = file_name[..file_name.len() - 7].to_string();
                    archive_path.push_str(&format!("{archive_index:03}.vpk"));
                    let file = fs::File::open(archive_path).map_err(|err| {
                        crate::reader::Error::Io("failed to read archive file", err)
                    })?;
                    let stm = BufReader::new(file);
                    dat_files.insert(archive_index, stm);
                }
            }
        }

        let file_data_offset = header_size + u64::from(hdr.file_tree_size);

        Ok(Self {
            dir_file: stm,
            dat_files,
            files,
            file_data_offset,
        })
    }
}

impl crate::reader::Reader for Reader {
    fn len(&self) -> usize {
        self.files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        let file = self.files.get(index).unwrap();
        let size = u64::try_from(file.preload_bytes.len()).unwrap() + u64::from(file.entry_offset);

        crate::reader::File {
            name: file.name.clone(),
            file_type: FileType::RegularFile,
            size: Some(size),
        }
    }

    fn open_file_by_index<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn Read + 'a>> {
        let file = self.files.get(index).unwrap();
        let mut output = file.preload_bytes.clone();

        if file.entry_length > 0 {
            let (stm, entry_offset) = match file.archive_index {
                Some(archive_index) => (
                    self.dat_files.get_mut(&archive_index).unwrap(),
                    u64::from(file.entry_offset),
                ),
                None => (
                    &mut self.dir_file,
                    self.file_data_offset + u64::from(file.entry_offset),
                ),
            };

            let mut entry_buf = vec![0; file.entry_length as usize];

            stm.seek(SeekFrom::Start(entry_offset))
                .map_err(crate::reader::Error::ReadingInputFile)?;
            stm.read_exact(&mut entry_buf)
                .map_err(crate::reader::Error::ReadingInputFile)?;

            output.append(&mut entry_buf);
        }

        let c = Cursor::new(output);
        Ok(Box::new(c))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
