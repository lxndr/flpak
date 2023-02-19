use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind, Result},
    path::{Component, Path, PathBuf},
};

use crate::FileType;
use glob::Pattern;
use walkdir::WalkDir;

pub struct InputFile {
    pub host_path: PathBuf,
    pub path: PathBuf,
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
            Error::new(
                // FIXME: ErrorKind::InvalidFilename,
                ErrorKind::Other,
                format!("failed to resolve directory path: {err}"),
            )
        })?;

        for entry_result in WalkDir::new(&dir) {
            let entry = entry_result?;
            let host_path = entry.into_path();
            let path = host_path
                .strip_prefix(&dir)
                .expect("should be able to strip path prefix");

            if path.to_string_lossy() == "" {
                continue;
            }

            if host_path.is_file() {
                self.file_list.insert(
                    path.to_path_buf(),
                    InputFile {
                        path: path.to_path_buf(),
                        host_path,
                        file_type: FileType::RegularFile,
                    },
                );
            } else if host_path.is_dir() {
                self.file_list.insert(
                    path.to_path_buf(),
                    InputFile {
                        path: path.to_path_buf(),
                        host_path,
                        file_type: FileType::Directory,
                    },
                );
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("failed to add {0}: invalid file type", host_path.display()),
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

pub trait IntoUnixPath {
    fn into_unix_path(&self) -> String;
}

impl IntoUnixPath for Path {
    fn into_unix_path(&self) -> String {
        let mut components = Vec::new();

        for cmp in self.components() {
            match cmp {
                Component::Normal(name) => {
                    let utf8_name = name
                        .to_str()
                        .expect("should be able to convert file name to utf8 string");
                    components.push(utf8_name);
                }
                _ => {
                    panic!("only normal path components are allowed");
                }
            }
        }

        components.join("/")
    }
}

#[cfg(test)]
mod tests {
    use crate::{FileType, IntoUnixPath};
    use std::path::Path;

    #[test]
    fn input_file_list_builder() {
        let files = super::InputFileListBuilder::new()
            .add_dir(std::path::Path::new("./samples/unpacked"))
            .unwrap()
            .build();

        assert_eq!(files[0].file_type, FileType::Directory);
        assert_eq!(files[0].path, Path::new("dir1"));
    }

    #[test]
    fn into_unix_path() {
        let path = Path::new("dir1/file1.txt");
        let unix_path = path.into_unix_path();
        assert_eq!(unix_path, "dir1/file1.txt");
    }
}
