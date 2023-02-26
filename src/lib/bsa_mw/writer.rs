use std::{
    collections::BTreeMap,
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use super::{
    hash::Hash,
    records::{FileRecord, Header},
};
use crate::{writer, FileType, InputFileList, WriteEx, PathBufUtils};

struct File {
    local_path: PathBuf,
    archive_path: String,
    size: u32,
    offset: u32,
    hash: Hash,
}

pub fn create_archive(
    files: InputFileList,
    path: &Path,
    _params: &HashMap<String, String>,
) -> writer::Result<()> {
    let input_files = collect_file_info(&files)?;
    let input_files_by_hash = create_hash_map(&input_files)?;
    let file_count = input_files.len();

    // file names and hashes
    let mut names = String::new();
    let mut name_offsets = Vec::with_capacity(file_count);
    let mut hash_buffer = Vec::with_capacity(file_count);

    for &input_file in input_files_by_hash.values() {
        let name_offset: u32 = names.len().try_into().expect("should fit into `u32`");
        name_offsets.push(name_offset);
        names.push_str(&input_file.archive_path.replace('/', "\\"));
        names.push('\0');
        hash_buffer.push(u64::from(&input_file.hash));
    }

    let mut out = fs::File::create(path).map_err(writer::Error::CreatingOutputFile)?;

    let hash_table_offset = u32::try_from((file_count * 8) + (file_count * 4) + names.len())
        .map_err(|_| writer::Error::Other("total size of file records exceeds 4GiB".into()))?;
    Header::write(&mut out, hash_table_offset, file_count).map_err(writer::Error::WritingHeader)?;

    for &input_file in input_files_by_hash.values() {
        FileRecord::write(&mut out, input_file.size, input_file.offset)
            .map_err(writer::Error::WritingFileIndex)?;
    }

    out.write_u32_le_vec(&name_offsets)
        .map_err(writer::Error::WritingFileIndex)?;
    out.write_all(names.as_bytes())
        .map_err(writer::Error::WritingFileIndex)?;
    out.write_u64_le_vec(&hash_buffer)
        .map_err(writer::Error::WritingFileIndex)?;

    for input_file in &input_files {
        let mut file = fs::File::open(&input_file.local_path)
            .map_err(|err| writer::Error::OpeningInputFile(input_file.local_path.clone(), err))?;
        io::copy(&mut file, &mut out)
            .map_err(|err| writer::Error::ArchivingInputFile(input_file.local_path.clone(), err))?;
    }

    Ok(())
}

fn collect_file_info(files: &InputFileList) -> writer::Result<Vec<File>> {
    let mut input_files = Vec::with_capacity(files.len());
    let mut file_data_offset: u32 = 0;

    for file in files.iter() {
        if file.file_type == FileType::RegularFile {
            let metadata = file.src_path.metadata().map_err(|err| {
                writer::Error::ReadingInputFileMetadata(file.src_path.clone(), err)
            })?;
            let size = u32::try_from(metadata.len())
                .map_err(|_| writer::Error::InputFileLarger4GiB(file.src_path.clone()))?;
            let path = file.dst_path.to_unix().to_ascii_lowercase();

            if !path.is_ascii() {
                return Err(writer::Error::InputFileNotAscii(path));
            }

            let hash = Hash::from_path(&path);

            input_files.push(File {
                local_path: file.src_path.clone(),
                archive_path: path,
                size,
                offset: file_data_offset,
                hash,
            });

            match file_data_offset.checked_add(size) {
                Some(sum) => file_data_offset = sum,
                None => {
                    return Err(writer::Error::TotalInputLarger4GiB);
                }
            }
        }
    }

    input_files.sort_unstable_by(|a, b| a.archive_path.cmp(&b.archive_path));

    Ok(input_files)
}

/// Creates a sorting_hash->file map. This sorts files by hash and checks hash collisions.
fn create_hash_map(files: &[File]) -> writer::Result<BTreeMap<u64, &File>> {
    let mut map = BTreeMap::new();

    for file in files.iter() {
        // file records must be sorted by hash. hashes are sorted first by lower four bytes, then by higher four bytes.
        let sorting_hash = (u64::from(file.hash.low) << 32) | u64::from(file.hash.high);

        if let Some(existing_file) = map.insert(sorting_hash, file) {
            return Err(writer::Error::InputFileDuplicateHash {
                file: file.archive_path.clone(),
                existing_file: existing_file.archive_path.clone(),
                hash: file.hash.to_string(),
            });
        }
    }

    Ok(map)
}
