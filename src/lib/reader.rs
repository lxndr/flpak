use std::{collections::HashMap, io, result};

use crate::FileType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open file: {0}")]
    OpeningInputFile(#[source] io::Error),

    #[error("failed to read file metadata: {0}")]
    ReadingInputFileMetadata(#[source] io::Error),

    #[error("failed to read file: {0}")]
    ReadingInputFile(#[source] io::Error),

    #[error("failed to read file signature: {0}")]
    ReadingSignature(#[source] io::Error),

    #[error("invalid signature {signature:X?}, expected {expected_signature:X?}")]
    InvalidSignature {
        signature: Vec<u8>,
        expected_signature: Vec<u8>,
    },

    #[error("invalid signature '{signature:}', expected '{expected_signature:}'")]
    InvalidStringSignature {
        signature: String,
        expected_signature: &'static str,
    },

    #[error("failed to read file header: {0}")]
    ReadingHeader(#[source] io::Error),

    #[error("invalid header: {0}")]
    InvalidHeader(String),

    #[error("invalid version {version}, expected one of {supported_versions:?}")]
    UnsupportedVersion {
        version: u32,
        supported_versions: &'static [u32],
    },

    #[error("invalid file name hash '{hash}' for '{filename}', expected '{expected_hash}'")]
    InvalidFileNameHash {
        filename: String,
        hash: String,
        expected_hash: String,
    },

    #[error("failed to read file index: {0}")]
    ReadingFileIndex(#[source] io::Error),

    #[error("failed to parse file (directory) name: {0}")]
    ReadingFileName(#[source] io::Error),

    #[error("not a regular file")]
    NotFile,

    #[error("{0}")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = result::Result<T, Error>;

pub struct File {
    pub name: String,
    pub file_type: FileType,
    pub size: Option<u64>,
}

#[derive(Default)]
pub struct Options {
    pub strict: bool,
}

pub trait Reader {
    fn file_count(&self) -> usize;
    fn get_file(&self, index: usize) -> File;
    fn create_file_reader<'a>(&'a mut self, index: usize) -> Result<Box<dyn io::Read + 'a>>;

    fn attrs(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
