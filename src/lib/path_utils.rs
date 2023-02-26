use std::{
    io::Result,
    path::{Component, PathBuf},
    str,
};

use crate::io_error;

pub trait PathBufUtils {
    fn from_win(path: &str) -> PathBuf;
    fn try_from_ascii_win(path: &str) -> Result<PathBuf>;
    fn try_to_win(&self) -> Result<String>;
    fn try_to_ascii_win(&self) -> Result<String>;

    fn from_unix(path: &str) -> PathBuf;
    fn to_unix(&self) -> String;
}

impl PathBufUtils for PathBuf {
    fn from_win(path: &str) -> PathBuf {
        let components = path.split('\\').collect::<Vec<&str>>();
        let mut path = PathBuf::new();

        for cmp in components {
            path.push(cmp);
        }

        path
    }

    fn try_from_ascii_win(path: &str) -> Result<PathBuf> {
        if !path.is_ascii() {
            return Err(io_error!(InvalidInput, "path is not ascii"));
        }

        Ok(Self::from_win(path))
    }

    fn try_to_win(&self) -> Result<String> {
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

        Ok(components.join("\\"))
    }

    fn try_to_ascii_win(&self) -> Result<String> {
        let path = self.try_to_win()?;

        if !path.is_ascii() {
            return Err(io_error!(InvalidInput, "path is not ascii"));
        }

        Ok(path)
    }

    fn from_unix(path: &str) -> PathBuf {
        let components = path.split('/').collect::<Vec<&str>>();
        let mut path = PathBuf::new();

        for cmp in components {
            path.push(cmp);
        }

        path
    }

    fn to_unix(&self) -> String {
        let mut components = Vec::new();

        for cmp in self.components() {
            match cmp {
                Component::Normal(name) => {
                    let utf8_name = name
                        .to_str()
                        .expect("should be able to convert file name to utf-8 string");
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
