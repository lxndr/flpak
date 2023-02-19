use std::{fs, io, path::Path};

use crate::FileType;
use zip::ZipArchive;

pub struct Reader {
    zip: ZipArchive<fs::File>,
    files: Vec<crate::reader::File>,
}

impl Reader {
    fn open(path: &Path, _options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut zip =
            ZipArchive::new(file).map_err(|err| crate::reader::Error::Other(err.to_string()))?;

        let mut files = Vec::new();

        for index in 0..zip.len() {
            let file = zip
                .by_index(index)
                .map_err(|err| crate::reader::Error::Other(err.to_string()))?;

            let file_type = if file.is_dir() {
                FileType::Directory
            } else {
                FileType::RegularFile
            };

            files.push(crate::reader::File {
                name: file.name().to_string(),
                file_type,
                size: Some(file.size()),
            });
        }

        Ok(Self { zip, files })
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
            .expect("should be able to get file by index");

        crate::reader::File {
            name: file.name.clone(),
            file_type: file.file_type.clone(),
            size: file.size,
        }
    }

    fn create_file_reader<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn io::Read + 'a>> {
        let file = self
            .zip
            .by_index(index)
            .expect("should be able to get file by index");

        Ok(Box::new(file))
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
