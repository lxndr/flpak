use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{self, Seek, Write},
    path::{Path, PathBuf},
};

use libflate::zlib;

use crate::{writer, FileType, InputFileList, PathBufUtils, WriteEx};

use super::{
    write_file_index::{File, Folder},
    Flags, Hash, Header, Version, WriteFileIndex, WriteHeader, BSA_SIGNATURE,
};

pub fn create_archive(
    input_files: InputFileList,
    path: &Path,
    options: &HashMap<String, String>,
) -> writer::Result<()> {
    let mut hdr = Header::default();
    parse_options(options, &mut hdr)?;

    let folder_record_size = match hdr.version {
        Version::V103 | Version::V104 => 16,
        Version::V105 => 24,
    };

    let mut folders = collect_file_index(&input_files)?;
    let mut total_folder_name_length = 0;
    let mut names = String::new();
    let mut file_count = 0;
    let mut folder_offset: usize = 36 + folder_record_size * folders.len();

    for folder in &mut folders {
        total_folder_name_length += folder.name.len() + 1;
        folder.offset = folder_offset.try_into().expect("should fit into `u32`");

        for file in &folder.files {
            file_count += 1;
            names.push_str(&file.name);
            names.push('\0');
        }

        folder_offset += folder.files.len() * 16;

        if hdr.flags.contains(Flags::HAS_FOLDER_NAMES) {
            folder_offset += folder.name.len() + 2; // +1 for the null terminator, +1 for the folder name length
        }
    }

    hdr.folder_count = folders.len() as u32;
    hdr.file_count = file_count as u32;
    hdr.total_folder_name_length = total_folder_name_length as u32;
    hdr.total_file_name_length = names.len() as u32;

    let mut file_data_offset =
        // header
        36 +
        // folder records
        folders.len() * folder_record_size +
        // file records
        file_count * 16 +
        // file names
        names.len();

    if hdr.flags.contains(Flags::HAS_FOLDER_NAMES) {
        file_data_offset += total_folder_name_length + folders.len();
    }

    // allocate space for header and file index
    let mut output_file = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;
    output_file
        .set_len(file_data_offset as u64)
        .map_err(writer::Error::CreatingOutputFile)?;
    output_file
        .seek(io::SeekFrom::Start(file_data_offset as u64))
        .map_err(writer::Error::CreatingOutputFile)?;

    // write input file data
    for folder in &mut folders {
        for file in &mut folder.files {
            let mut input_file = fs::File::open(&file.local_path)
                .map_err(|err| writer::Error::OpeningInputFile(file.local_path.clone(), err))?;
            let input_file_size = input_file
                .metadata()
                .map_err(|err| writer::Error::OpeningInputFile(file.local_path.clone(), err))?
                .len();

            if hdr.flags.contains(Flags::EMBEDDED_FILE_NAMES) {
                output_file
                    .write_u8_string(&format!("{}/{}", folder.name, file.name))
                    .map_err(writer::Error::WritingFileData)?;
            }

            let output_file_pos = output_file
                .stream_position()
                .map_err(writer::Error::WritingFileData)?;
            file.offset = output_file_pos
                .try_into()
                .map_err(|_| writer::Error::OutputFileLarger4GiB)?;

            if hdr.flags.contains(Flags::COMPRESSED_BY_DEFAULT) {
                let input_file_size = input_file_size
                    .try_into()
                    .map_err(|_| writer::Error::InputFileLarger4GiB(file.local_path.clone()))?;
                output_file
                    .write_u32_le(input_file_size)
                    .map_err(writer::Error::WritingFileData)?;

                match hdr.version {
                    Version::V103 | Version::V104 => {
                        let mut encoder = zlib::Encoder::new(&mut output_file)
                            .map_err(writer::Error::WritingFileData)?;
                        io::copy(&mut input_file, &mut encoder)
                            .map_err(writer::Error::WritingFileData)?;
                        encoder
                            .finish()
                            .into_result()
                            .map_err(writer::Error::WritingFileData)?;
                    }
                    Version::V105 => {
                        let mut encoder = lz4_flex::frame::FrameEncoder::new(&mut output_file);
                        io::copy(&mut input_file, &mut encoder)
                            .map_err(writer::Error::WritingFileData)?;
                        encoder
                            .finish()
                            .map_err(|err| writer::Error::Other(err.to_string()))?;
                    }
                }
            } else {
                io::copy(&mut input_file, &mut output_file)
                    .map_err(writer::Error::WritingFileData)?;
            }

            let end_pos = output_file
                .stream_position()
                .map_err(writer::Error::WritingFileData)?;

            file.size = (end_pos - output_file_pos)
                .try_into()
                .map_err(|_| writer::Error::InputFileLarger4GiB(file.local_path.clone()))?;
        }
    }

    // write header and file index
    output_file
        .rewind()
        .map_err(writer::Error::WritingSignature)?;
    let mut wrt = io::BufWriter::new(output_file);
    wrt.write_all(BSA_SIGNATURE)
        .map_err(writer::Error::WritingHeader)?;
    wrt.write_header(&hdr)
        .map_err(writer::Error::WritingHeader)?;
    wrt.write_file_index(&folders, &names, &hdr)
        .map_err(writer::Error::WritingHeader)?;

    Ok(())
}

fn parse_options(params: &HashMap<String, String>, hdr: &mut Header) -> crate::writer::Result<()> {
    hdr.version = Version::try_from(params.get("version"))
        .map_err(|err| writer::Error::InvalidParameter("version", err))?;

    let compress = params.get("compress").map_or(false, |v| v == "true");
    let big_endian = params.get("big-endian").map_or(false, |v| v == "true");
    let xmem_codec = params.get("xmem-codec").map_or(false, |v| v == "true");

    if compress {
        hdr.flags |= Flags::COMPRESSED_BY_DEFAULT;
    }

    if big_endian {
        hdr.flags |= Flags::XBOX;
    }

    if xmem_codec {
        hdr.flags |= Flags::COMPRESSED_BY_DEFAULT;
        hdr.flags |= Flags::XMEM_CODEC;
    }

    if xmem_codec {
        return Err(writer::Error::Other(
            "XMEM codec is not supported yet".into(),
        ));
    }

    Ok(())
}

fn collect_file_index(input_files: &InputFileList) -> writer::Result<Vec<Folder>> {
    let mut folders: BTreeMap<String, Folder> = BTreeMap::new();

    for input_file in input_files {
        match input_file.file_type {
            FileType::Directory => {
                let folder_name = input_file
                    .dst_path
                    .try_to_win()
                    .map_err(|err| {
                        writer::Error::InvalidInputFileName(input_file.dst_path.clone(), err)
                    })?
                    .to_lowercase();
                add_folder(&mut folders, folder_name)?;
            }
            FileType::RegularFile => {
                let file_name = input_file
                    .dst_path
                    .file_name()
                    .expect("should get file name")
                    .to_str()
                    .expect("should convert file name to utf-8 string")
                    .to_lowercase();
                let folder_name = input_file
                    .dst_path
                    .parent()
                    .unwrap_or(&input_file.dst_path)
                    .to_path_buf()
                    .try_to_win()
                    .map_err(|err| {
                        writer::Error::InvalidInputFileName(input_file.dst_path.clone(), err)
                    })?
                    .to_lowercase();

                if folder_name.is_empty() {
                    return Err(writer::Error::InputFileNotInFolder(file_name));
                }

                if !file_name.is_ascii() {
                    return Err(writer::Error::InputFileNotAscii(folder_name));
                }

                let folder = add_folder(&mut folders, folder_name)?;

                folder.files.push(File {
                    local_path: input_file.src_path.clone(),
                    name_hash: Hash::from_file_name(&file_name),
                    name: file_name,
                    size: 0,
                    offset: 0,
                });
            }
        }
    }

    let mut folders: Vec<Folder> = folders.into_values().collect();

    // no empty folders
    folders.retain(|folder| !folder.files.is_empty());

    // folders should be sorted by name hash
    folders.sort_by(|a, b| a.name_hash.cmp(&b.name_hash));

    // files should be sorted by name hash
    for folder in &mut folders {
        folder.files.sort_by(|a, b| a.name_hash.cmp(&b.name_hash));
    }

    Ok(folders)
}

fn add_folder(
    folders: &mut BTreeMap<String, Folder>,
    folder_name: String,
) -> writer::Result<&mut Folder> {
    if !folder_name.is_ascii() {
        return Err(writer::Error::InputFileNotAscii(folder_name));
    }

    if folder_name.len() > 255 {
        return Err(writer::Error::InputFileNameTooLong(
            folder_name.clone(),
            folder_name.len(),
        ));
    }

    let entry_key = folder_name.clone();

    let folder = folders.entry(entry_key).or_insert_with(|| Folder {
        name_hash: Hash::from_folder_path(&PathBuf::from_win(&folder_name)),
        name: folder_name,
        offset: 0,
        files: Vec::new(),
    });

    Ok(folder)
}
