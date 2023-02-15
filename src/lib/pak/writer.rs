use std::{
    fs,
    io::{self, Cursor, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{writer, FileType, InputFile, InputFileList, IntoUnixPath, WriteEx};

use super::common::{PAK_FILE_ENTRY_SIZE, PAK_SIGNATURE};

pub fn create_archive(input_files: InputFileList, path: &Path) -> writer::Result<()> {
    let mut out = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;

    out.write_all(PAK_SIGNATURE)
        .map_err(writer::Error::WritingHeader)?;

    // header placeholder
    out.seek(SeekFrom::Start(12))
        .map_err(writer::Error::WritingHeader)?;

    // files and making file index
    let input_files: Vec<&InputFile> = input_files
        .iter()
        .filter(|f| f.file_type == FileType::RegularFile)
        .collect();
    let mut index_buffer = vec![0u8; input_files.len() * PAK_FILE_ENTRY_SIZE];
    let mut index_cursor = Cursor::new(&mut index_buffer);

    for input_file in input_files {
        let metadata = input_file.host_path.metadata().map_err(|err| {
            writer::Error::ReadingInputFileMetadata(input_file.host_path.clone(), err)
        })?;
        let size = u32::try_from(metadata.len())
            .map_err(|_| writer::Error::InputFileLarger4GiB(input_file.host_path.clone()))?;
        let path = input_file.path.into_unix_path();

        if path.len() > 55 {
            return Err(writer::Error::InputFileNameTooLong(55));
        }

        let offset = out
            .stream_position()
            .map_err(|err| writer::Error::ArchivingInputFile(input_file.host_path.clone(), err))?;

        let mut file = fs::File::open(&input_file.host_path)
            .map_err(|err| writer::Error::OpeningInputFile(input_file.host_path.clone(), err))?;

        io::copy(&mut file, &mut out)
            .map_err(|err| writer::Error::ArchivingInputFile(input_file.host_path.clone(), err))?;

        let offset = u32::try_from(offset)
            .map_err(|_| writer::Error::InputFileLarger4GiB(input_file.host_path.clone()))?;
        let size = u32::try_from(size)
            .map_err(|_| writer::Error::InputFileLarger4GiB(input_file.host_path.clone()))?;

        index_cursor
            .write_all(format!("{:\0<56}", path).as_bytes())
            .expect("writing to memory buffer");
        index_cursor
            .write_u32_le(offset)
            .expect("writing to memory buffer");
        index_cursor
            .write_u32_le(size)
            .expect("writing to memory buffer");
    }

    // encode, compress file index
    let index_offset = out
        .stream_position()
        .map_err(writer::Error::WritingFileIndex)?;
    let index_offset =
        u32::try_from(index_offset).map_err(|_| writer::Error::OutputFileLarger4GiB)?;
    let index_size =
        u32::try_from(index_buffer.len()).map_err(|_| writer::Error::OutputFileLarger4GiB)?;

    out.write_all(&index_buffer)
        .map_err(writer::Error::WritingFileIndex)?;

    // write real header
    out.seek(SeekFrom::Start(4))
        .map_err(writer::Error::WritingHeader)?;
    out.write_u32_le(index_offset)
        .map_err(writer::Error::WritingHeader)?;
    out.write_u32_le(index_size)
        .map_err(writer::Error::WritingHeader)?;

    Ok(())
}
