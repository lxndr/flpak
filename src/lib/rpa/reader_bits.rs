use std::{
    collections::BTreeMap,
    io::{BufRead, Read, Result},
    path::PathBuf,
    str,
};

use libflate::zlib;

use crate::{io_error, PathBufUtils};

pub struct Header {
    pub signature: String,
    pub index_offset: u64,
    pub key: u64,
}

impl Header {
    #[inline]
    pub fn read(r: &mut impl BufRead) -> Result<Self> {
        let mut header = String::new();
        r.read_line(&mut header)?;

        let parts: Vec<&str> = header[..header.len() - 1].split(' ').collect();

        if parts.len() != 3 {
            return Err(io_error!(InvalidData, "invalid header"));
        }

        let signature = parts[0].to_string();

        let Ok(index_offset) = u64::from_str_radix(parts[1], 16) else {
            return Err(io_error!(InvalidData, "invalid index offset"))
        };

        let Ok(key) = u64::from_str_radix(parts[2], 16) else {
            return Err(io_error!(InvalidData, "invalid key"))
        };

        Ok(Header {
            signature,
            index_offset,
            key,
        })
    }
}

pub struct File {
    pub name: PathBuf,
    pub size: u64,
    pub offset: u64,
}

type FileIndex = BTreeMap<String, Vec<(u64, u64, String)>>;

pub fn read_file_index(r: &mut impl BufRead, key: u64) -> Result<Vec<File>> {
    let mut compressed_index = Vec::new();
    r.read_to_end(&mut compressed_index)?;

    let mut decompressed_index = Vec::new();
    let mut decoder = zlib::Decoder::new(&compressed_index[..])?;
    decoder.read_to_end(&mut decompressed_index)?;
    let deserialized: FileIndex =
        serde_pickle::from_slice(&decompressed_index, serde_pickle::DeOptions::default()).unwrap();

    let files = deserialized
        .iter()
        .map(|(name, entries)| {
            let (offset, size, _) = entries[0];

            File {
                name: PathBuf::from_unix(name),
                offset: offset ^ key,
                size: size ^ key,
            }
        })
        .collect();

    Ok(files)
}
