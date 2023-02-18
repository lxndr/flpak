use std::{fs, io, path::Path};

use crate::{writer, FileType, InputFileList, IntoUnixPath};

pub fn create_archive(input_files: InputFileList, path: &Path) -> writer::Result<()> {
    let out = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;
    let mut zip = zip::ZipWriter::new(out);

    for input_file in input_files {
        match input_file.file_type {
            FileType::Directory => {
                let path = format!("{}/", input_file.path.into_unix_path());
                zip.add_directory(path, Default::default()).map_err(|err| {
                    writer::Error::WritingFileData(io::Error::new(
                        io::ErrorKind::Other,
                        err.to_string(),
                    ))
                })?;
            }
            FileType::RegularFile => {
                let mut file = fs::File::open(&input_file.host_path).map_err(|err| {
                    writer::Error::OpeningInputFile(input_file.host_path.clone(), err)
                })?;

                let path = input_file.path.into_unix_path();
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);

                zip.start_file(path, options).map_err(|err| {
                    writer::Error::WritingFileData(io::Error::new(
                        io::ErrorKind::Other,
                        err.to_string(),
                    ))
                })?;

                io::copy(&mut file, &mut zip).map_err(|err| {
                    writer::Error::WritingFileData(io::Error::new(
                        io::ErrorKind::Other,
                        err.to_string(),
                    ))
                })?;
            }
        }
    }

    zip.finish()
        .map_err(|err| writer::Error::Other(err.to_string()))?;

    Ok(())
}
