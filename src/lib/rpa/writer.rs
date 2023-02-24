use std::{
    collections::HashMap,
    fs,
    io::{self, Seek, Write},
    path::Path,
};

use super::common::{FileIndex, DEFAULT_KEY, RENPY_PADDING};
use crate::{writer, FileType, InputFileList, ToUnixPath};
use libflate::zlib;

pub fn create_archive(
    input_files: InputFileList,
    path: &Path,
    _params: &HashMap<String, String>,
) -> writer::Result<()> {
    let mut out = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;
    let mut file_index = FileIndex::new();

    // header placeholder
    let header = format!("RPA-3.0 {:016x} {DEFAULT_KEY:08x}\n", 0);
    out.write_all(header.as_bytes())
        .map_err(writer::Error::WritingHeader)?;

    // files and making file index
    for input_file in input_files {
        if input_file.file_type == FileType::RegularFile {
            let metadata = input_file.host_path.metadata().map_err(|err| {
                writer::Error::ReadingInputFileMetadata(input_file.host_path.clone(), err)
            })?;
            let size = metadata.len();
            let path = input_file.path.to_unix_path();

            out.write_all(RENPY_PADDING).map_err(|err| {
                writer::Error::ArchivingInputFile(input_file.host_path.clone(), err)
            })?;
            let offset = out.stream_position().map_err(|err| {
                writer::Error::ArchivingInputFile(input_file.host_path.clone(), err)
            })?;

            let mut file = fs::File::open(&input_file.host_path).map_err(|err| {
                writer::Error::OpeningInputFile(input_file.host_path.clone(), err)
            })?;
            io::copy(&mut file, &mut out).map_err(|err| {
                writer::Error::ArchivingInputFile(input_file.host_path.clone(), err)
            })?;

            file_index.insert(
                path,
                vec![(
                    offset ^ u64::from(DEFAULT_KEY),
                    size ^ u64::from(DEFAULT_KEY),
                    String::from(""),
                )],
            );
        }
    }

    // encode, compress file index
    let file_index_offset = out
        .stream_position()
        .map_err(writer::Error::WritingFileIndex)?;

    let mut zlib_encoder = zlib::Encoder::new(&mut out).expect("should create zlib encoder");
    serde_pickle::to_writer(
        &mut zlib_encoder,
        &file_index,
        serde_pickle::SerOptions::new(),
    )
    .map_err(|err| writer::Error::WritingFileIndexCustom(err.to_string()))?;

    zlib_encoder
        .finish()
        .into_result()
        .map_err(writer::Error::WritingFileIndex)?;

    // write real header
    let header = format!("RPA-3.0 {file_index_offset:016x} {DEFAULT_KEY:08x}\n");
    out.rewind().map_err(writer::Error::WritingHeader)?;
    out.write_all(header.as_bytes())
        .map_err(writer::Error::WritingHeader)?;

    Ok(())
}
