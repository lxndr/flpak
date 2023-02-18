use std::{io, path::PathBuf, result};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create output file: {0}")]
    CreatingOutputFile(#[source] io::Error),

    #[error("failed to write archive signature: {0}")]
    WritingSignature(#[source] io::Error),

    #[error("failed to write archive header: {0}")]
    WritingHeader(#[source] io::Error),

    #[error("failed to write file records: {0}")]
    WritingFileIndex(#[source] io::Error),

    #[error("failed to write file records: {0}")]
    WritingFileIndexCustom(String),

    #[error("failed to write file data: {0}")]
    WritingFileData(#[source] io::Error),

    #[error("failed to open input file '{0}': {0}")]
    OpeningInputFile(PathBuf, #[source] io::Error),

    #[error("failed to read input file '{0}' metadata: {0}")]
    ReadingInputFileMetadata(PathBuf, #[source] io::Error),

    #[error("failed to archive file '{0}': {0}")]
    ArchivingInputFile(PathBuf, #[source] io::Error),

    #[error("failed to archive file '{0}: file size should not be larger than 4 GiB")]
    InputFileLarger4GiB(PathBuf),

    #[error("failed to archive file '{0}: file name should not be longer than {0}")]
    InputFileNameTooLong(usize),

    #[error("failed to archive file '{0}: file name can only contain ascii characters")]
    InputFileNotAscii(String),

    #[error("failed to archive file '{file}': duplicate hash {hash} for file {existing_file}")]
    InputFileDuplicateHash {
        file: String,
        existing_file: String,
        hash: String,
    },

    #[error("total size of files cannot be larger than 4GiB")]
    TotalInputLarger4GiB,

    #[error("output file cannot be larger than 4GiB")]
    OutputFileLarger4GiB,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = result::Result<T, Error>;
