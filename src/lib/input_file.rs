use std::{
    collections::BTreeMap,
    io::Result,
    path::{Path, PathBuf},
};

use crate::{io_error, FileType};
use glob::Pattern;
use walkdir::WalkDir;

pub struct InputFile {
    pub src_path: PathBuf,
    pub dst_path: PathBuf,
    pub file_type: FileType,
}

pub type InputFileList = Vec<InputFile>;

pub struct InputFileListBuilder {
    file_list: BTreeMap<PathBuf, InputFile>,
}

impl InputFileListBuilder {
    pub fn new() -> Self {
        Self {
            file_list: BTreeMap::new(),
        }
    }

    pub fn build(self) -> Vec<InputFile> {
        self.file_list.into_values().collect()
    }

    pub fn add_dir(mut self, dir: &Path) -> Result<Self> {
        let dir = dir.canonicalize().map_err(|err| {
            io_error!(
                // FIXME: ErrorKind::InvalidFilename,
                Other,
                "failed to resolve directory path: {err}",
            )
        })?;

        for entry_result in WalkDir::new(&dir) {
            let entry = entry_result?;
            let src_path = entry.into_path();
            let path = src_path
                .strip_prefix(&dir)
                .expect("should be able to strip path prefix");

            if path.to_string_lossy() == "" {
                continue;
            }

            if src_path.is_file() {
                self.file_list.insert(
                    path.to_path_buf(),
                    InputFile {
                        dst_path: path.to_path_buf(),
                        src_path,
                        file_type: FileType::RegularFile,
                    },
                );
            } else if src_path.is_dir() {
                self.file_list.insert(
                    path.to_path_buf(),
                    InputFile {
                        dst_path: path.to_path_buf(),
                        src_path,
                        file_type: FileType::Directory,
                    },
                );
            } else {
                return Err(io_error!(
                    InvalidInput,
                    "failed to add {0}: invalid file type",
                    src_path.display(),
                ));
            }
        }

        Ok(self)
    }

    pub fn exclude_pattern(mut self, pattern: &str) -> Self {
        let pattern = Pattern::new(pattern).unwrap();
        self.file_list.retain(|path, _| !pattern.matches_path(path));
        self
    }
}

impl Default for InputFileListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FileType, PathBufUtils};
    use std::path::Path;

    #[test]
    fn input_file_list_builder() {
        let files = super::InputFileListBuilder::new()
            .add_dir(std::path::Path::new("./samples/unpacked"))
            .unwrap()
            .build();

        assert_eq!(files[0].file_type, FileType::Directory);
        assert_eq!(files[0].dst_path, Path::new("dir1"));
    }

    #[test]
    fn into_unix_path() {
        let path = Path::new("dir1/file1.txt");
        let unix_path = path.to_path_buf().try_to_unix().unwrap();
        assert_eq!(unix_path, "dir1/file1.txt");
    }
}
