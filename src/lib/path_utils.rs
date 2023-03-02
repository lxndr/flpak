use std::{
    io::Result,
    path::{Component, PathBuf},
    str,
};

use crate::io_error;

pub trait PathBufUtils {
    fn is_safe(&self) -> bool;

    fn from_win(path: &str) -> PathBuf;
    fn try_to_win(&self) -> Result<String>;

    fn from_unix(path: &str) -> PathBuf;
    fn try_to_unix(&self) -> Result<String>;

    fn to_safe_components(&self) -> Result<Vec<&str>>;
}

impl PathBufUtils for PathBuf {
    fn is_safe(&self) -> bool {
        for cmp in self.components() {
            match cmp {
                Component::Normal(_) => (),
                _ => return false,
            }
        }

        true
    }

    fn from_win(path: &str) -> PathBuf {
        let components = path.split('\\').collect::<Vec<&str>>();
        let mut path = PathBuf::new();

        for cmp in components {
            path.push(cmp);
        }

        path
    }

    fn try_to_win(&self) -> Result<String> {
        Ok(self.to_safe_components()?.join("\\"))
    }

    fn from_unix(path: &str) -> PathBuf {
        let components = path.split('/').collect::<Vec<&str>>();
        let mut path = PathBuf::new();

        for cmp in components {
            path.push(cmp);
        }

        path
    }

    fn try_to_unix(&self) -> Result<String> {
        Ok(self.to_safe_components()?.join("/"))
    }

    fn to_safe_components(&self) -> Result<Vec<&str>> {
        let mut components = Vec::new();

        for cmp in self.components() {
            match cmp {
                Component::Normal(name) => {
                    let utf8_name = name
                        .to_str()
                        .ok_or_else(|| io_error!(InvalidInput, "file name is not utf-8"))?;
                    components.push(utf8_name);
                }
                _ => {
                    return Err(io_error!(
                        InvalidInput,
                        "only normal path components are allowed",
                    ));
                }
            }
        }

        Ok(components)
    }
}
