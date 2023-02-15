use std::{io, result};

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

    #[error("invalid signature {signature:?}, expected {expected_signature:?}")]
    InvalidSignature {
        signature: [u8; 4],
        expected_signature: &'static [u8; 4],
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

    #[error("not a regular file")]
    NotFile,
    #[error("{0}")]
    Unsupported(&'static str),
    #[error("{0}: {1}")]
    Io(&'static str, io::Error),
    #[error("{0}")]
    Other(&'static str),
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
    fn len(&self) -> usize;
    fn get_file(&self, index: usize) -> File;
    fn open_file_by_index<'a>(&'a mut self, index: usize) -> Result<Box<dyn io::Read + 'a>>;
}
