use std::{collections::HashMap, fs, io, path::Path};

use crate::{io_error, writer, FileType, InputFileList, PathBufUtils};

pub fn create_archive(
    input_files: InputFileList,
    path: &Path,
    _params: &HashMap<String, String>,
) -> writer::Result<()> {
    let out = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;
    let mut zip = zip::ZipWriter::new(out);

    for input_file in input_files {
        match input_file.file_type {
            FileType::Directory => {
                let path = format!("{}/", input_file.dst_path.to_unix());
                zip.add_directory(path, Default::default())
                    .map_err(|err| writer::Error::WritingFileData(io_error!(Other, "{}", err,)))?;
            }
            FileType::RegularFile => {
                let mut file = fs::File::open(&input_file.src_path).map_err(|err| {
                    writer::Error::OpeningInputFile(input_file.src_path.clone(), err)
                })?;

                let path = input_file.dst_path.to_unix();
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);

                zip.start_file(path, options)
                    .map_err(|err| writer::Error::WritingFileData(io_error!(Other, "{}", err,)))?;

                io::copy(&mut file, &mut zip)
                    .map_err(|err| writer::Error::WritingFileData(io_error!(Other, "{}", err,)))?;
            }
        }
    }

    zip.finish()
        .map_err(|err| writer::Error::Other(err.to_string()))?;

    Ok(())
}
