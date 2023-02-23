use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
    result,
};

use crate::{ba2, bsa, bsa_mw, pak, reader, rpa, vpk, writer, zip, InputFileList};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown format '{0}'")]
    UnknownFormat(String),
    #[error("unable to detect format by signature")]
    UnableToDetect,
    #[error("reading '{0}' not supported")]
    ReadingUnsupported(String),
    #[error("creating '{0}' not supported")]
    CreatingUnsupported(String),
    #[error("{0}")]
    ReaderError(reader::Error),
    #[error("{0}")]
    IoError(io::Error),
}

pub type Result<T> = result::Result<T, Error>;
pub type MakeReaderFn =
    fn(path: &Path, options: reader::Options) -> reader::Result<Box<dyn reader::Reader>>;
pub type WriterFn =
    fn(files: InputFileList, path: &Path, params: HashMap<String, String>) -> writer::Result<()>;

pub struct FormatDesc {
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: Vec<&'static str>,
    pub signatures: Vec<Vec<u8>>,
    pub make_reader_fn: Option<MakeReaderFn>,
    pub writer_fn: Option<WriterFn>,
}

pub struct Registry {
    formats: Vec<FormatDesc>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        let formats = vec![
            FormatDesc {
                name: "bsa-mw",
                description: "Bethesda Archive (v100)",
                extensions: vec!["bsa"],
                signatures: vec![vec![0x00, 0x01, 0x00, 0x00]],
                make_reader_fn: Some(bsa_mw::make_reader),
                writer_fn: Some(bsa_mw::create_archive),
            },
            FormatDesc {
                name: "bsa",
                description: "Bethesda Archive (v103, v104, v105)",
                extensions: vec!["bsa"],
                signatures: vec![b"BSA\0".to_vec()],
                make_reader_fn: Some(bsa::make_reader),
                writer_fn: Some(bsa::create_archive),
            },
            FormatDesc {
                name: "ba2",
                description: "Bethesda Archive 2",
                extensions: vec!["ba2"],
                signatures: vec![b"BTDX".to_vec()],
                make_reader_fn: Some(ba2::make_reader),
                writer_fn: None,
            },
            FormatDesc {
                name: "pak",
                description: "id Software .pak",
                extensions: vec!["vpk"],
                signatures: vec![b"PACK".to_vec()],
                make_reader_fn: Some(pak::make_reader),
                writer_fn: Some(pak::create_archive),
            },
            FormatDesc {
                name: "rpa",
                description: "Ren'Py Archive",
                extensions: vec!["rpa"],
                signatures: vec![b"RPA-".to_vec()],
                make_reader_fn: Some(rpa::make_reader),
                writer_fn: Some(rpa::create_archive),
            },
            FormatDesc {
                name: "vpk",
                description: "Valve Pack",
                extensions: vec!["vpk"],
                signatures: vec![vec![0x34, 0x12, 0xAA, 0x55]],
                make_reader_fn: Some(vpk::make_reader),
                writer_fn: None,
            },
            FormatDesc {
                name: "zip",
                description: "ZIP",
                extensions: vec!["zip"],
                signatures: vec![
                    b"PK\x03\x04".to_vec(),
                    b"PK\x05\x06".to_vec(),
                    b"PK\x07\x08".to_vec(),
                ],
                make_reader_fn: Some(zip::make_reader),
                writer_fn: Some(zip::create_archive),
            },
        ];

        Self { formats }
    }

    #[must_use]
    pub fn list(&self) -> &Vec<FormatDesc> {
        &self.formats
    }

    #[must_use]
    pub fn create_reader(
        &self,
        format: Option<String>,
        path: &Path,
        options: reader::Options,
    ) -> Result<Box<dyn reader::Reader>> {
        let format_desc = if let Some(format) = format {
            let Some(format_desc) = self.find_format_by_name(&format) else {
                return Err(Error::UnknownFormat(format));
            };

            format_desc
        } else {
            let mut sig = vec![0u8; 4];
            let mut file = File::open(path).map_err(|err| Error::IoError(err))?;
            file.read_exact(&mut sig)
                .map_err(|err| Error::IoError(err))?;

            let Some(format_desc) = self.find_format_by_signature(&sig) else {
                return Err(Error::UnableToDetect);
            };

            format_desc
        };

        let Some(make_reader_fn) = format_desc.make_reader_fn else {
            return Err(Error::ReadingUnsupported(format_desc.name.to_string()));
        };

        make_reader_fn(path, options).map_err(|err| Error::ReaderError(err))
    }

    #[must_use]
    pub fn create_writer(&self, format: &str) -> Result<WriterFn> {
        let Some(format_desc) = self.find_format_by_name(format) else {
            return Err(Error::UnknownFormat(format.to_string()));
        };

        let Some(writer_fn) = format_desc.writer_fn else {
            return Err(Error::CreatingUnsupported(format_desc.name.to_string()));
        };

        Ok(writer_fn)
    }

    #[must_use]
    pub fn find_format_by_name(&self, name: &str) -> Option<&FormatDesc> {
        self.formats.iter().find(|f| f.name == name)
    }

    #[must_use]
    pub fn find_format_by_signature(&self, buf: &[u8]) -> Option<&FormatDesc> {
        self.formats
            .iter()
            .find(|f| f.signatures.iter().find(|&sig| sig == buf).is_some())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn new_registry() {
        let registry = super::Registry::new();
        let formats = registry.list();

        assert_eq!(formats.len(), 7);
    }

    #[test]
    fn create_reader() {
        let registry = super::Registry::new();
        let res = registry.create_reader(
            Some(String::from("bsa-mw")),
            Path::new("./samples/bsa-mw/correct.bsa"),
            crate::reader::Options { strict: false },
        );

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn create_reader_with_unknown_format() {
        let registry = super::Registry::new();
        let res = registry.create_reader(
            Some(String::from("nonexistent-format")),
            Path::new("./samples/bsa-mw/correct.bsa"),
            crate::reader::Options { strict: false },
        );

        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn create_reader_with_format_detection() {
        let registry = super::Registry::new();
        let res = registry.create_reader(
            None,
            Path::new("./samples/bsa-mw/correct.bsa"),
            crate::reader::Options { strict: false },
        );

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn create_writer() {
        let registry = super::Registry::new();
        let res = registry.create_writer("bsa-mw");
        assert_eq!(res.is_ok(), true);
    }
}
