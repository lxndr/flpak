use std::{fmt, io};

use crate::{ReadEx, WriteEx};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash(u64);

impl Hash {
    pub fn from_file_name(fname: &str) -> Hash {
        let (name, ext) = match fname.rfind('.') {
            Some(pos) => (&fname[..pos], &fname[pos..]),
            None => (fname, ""),
        };

        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len();
        let ext_bytes = ext.as_bytes();

        let mut hash1 = Self::calc_name_hash(name);
        hash1 |= Self::calc_ext_hash(ext);

        let hash2: u64 = if name_len > 3 {
            Self::calc_slice_hash(&name_bytes[1..name_len - 2])
                .wrapping_add(Self::calc_slice_hash(ext_bytes))
        } else {
            Self::calc_slice_hash(ext_bytes)
        };

        Hash((hash2 << 32) + u64::from(hash1))
    }

    pub fn from_folder_path(name: &str) -> Hash {
        let name = Self::normalize_path(name);
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len();

        let hash1 = Self::calc_name_hash(&name);
        let hash2: u64 = if name_len > 3 {
            Self::calc_slice_hash(&name_bytes[1..name_len - 2])
        } else {
            0
        };

        Hash((hash2 << 32) + u64::from(hash1))
    }

    fn normalize_path(path: &str) -> String {
        assert!(!path.is_empty(), "Path cannot be empty");
        path.to_ascii_lowercase().replace('/', "\\")
    }

    fn calc_name_hash(name: &str) -> u32 {
        let bytes = name.as_bytes();
        let len = bytes.len();

        let mut hash: u32 = u32::from(bytes[len - 1]);
        hash |= u32::from(if len < 3 { 0 } else { bytes[len - 2] }) << 8;
        hash |= (len as u32) << 16;
        hash |= u32::from(bytes[0]) << 24;

        hash
    }

    fn calc_ext_hash(ext: &str) -> u32 {
        match ext {
            ".kf" => 0x80,
            ".nif" => 0x8000,
            ".dds" => 0x8080,
            ".wav" => 0x80000000,
            _ => 0,
        }
    }

    fn calc_slice_hash(bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0;

        for &byte in bytes {
            hash = hash.wrapping_mul(0x1003f).wrapping_add(u64::from(byte));
        }

        hash
    }
}

impl From<u64> for Hash {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Hash> for u64 {
    fn from(val: Hash) -> Self {
        val.0
    }
}

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

pub trait ReadHash: io::BufRead {
    fn read_hash(&mut self, big_endian: bool) -> io::Result<Hash> {
        Ok(Hash(self.read_u64(big_endian)?))
    }
}

impl<R: io::BufRead + ?Sized> ReadHash for R {}

pub trait WriteHash: io::Write {
    fn write_hash(&mut self, hash: &Hash, big_endian: bool) -> io::Result<()> {
        self.write_u64(hash.0, big_endian)
    }
}

impl<R: io::Write + ?Sized> WriteHash for R {}
